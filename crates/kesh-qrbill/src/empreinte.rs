//! Le comparateur du rendu d'un PDF — Story 49-1b (#1242), décision D4.
//!
//! **Réservé aux tests** : ce module n'existe que sous la feature `test-support`, que la
//! crate active pour ses propres tests (auto-dev-dépendance) et que `kesh-api` active par
//! sa dev-dépendance. Il n'entre jamais dans `cargo build --release -p kesh-api`.
//!
//! Il répond à une seule question : *ce PDF dessine-t-il exactement la même chose que
//! celui-là ?* Les octets du fichier ne le disent pas : `printpdf` tire au hasard le second
//! élément du `/ID` du trailer et l'identifiant d'instance de la métadonnée XMP
//! (`pdf_document.rs:793`, `xmp_metadata.rs:42` de printpdf 0.7), la date de création
//! entre dans `Info` et la XMP, et une construction `release` élague les objets
//! (`PdfDocument::optimize`) quand une construction de test ne le fait pas. L'**empreinte**
//! d'un PDF est donc faite de ce qui se dessine :
//!
//! - le **condensat SHA-256 de la suite complète des opérations de contenu** de chaque page,
//!   dans l'ordre (opérateur et opérandes : textes, positions, polices et tailles, traits,
//!   rectangles du QR, couleurs, épaisseurs) ;
//! - le **dictionnaire complet de chaque police** (`BaseFont`, `Encoding`, `Subtype`…) ;
//! - le dictionnaire des **ressources** de chaque page, références suivies ;
//! - la **`MediaBox`** de chaque page et le **nombre de pages** ;
//! - en lisible, la liste `(x, y, texte)` des textes, pour qu'un écart **se lise**.
//!
//! Le dictionnaire `Info` (dates, producteur), la XMP et le trailer sont **exclus**, et le
//! disent : ils ne se dessinent pas.
//!
//! ⛔ Le comparateur est **gelé** avec les références de la 0.19.0 (fiche 49-1b, AC1 (6)) :
//! le modifier après leur capture rendrait la preuve avant / après invérifiable.

use std::fmt::Write as _;
use std::path::Path;

use printpdf::lopdf::content::{Content, Operation};
use printpdf::lopdf::{Dictionary, Document, Object, StringFormat};
use sha2::{Digest, Sha256};

/// Millimètres par point typographique.
const MM_PAR_POINT: f32 = 25.4 / 72.0;

/// Profondeur au-delà de laquelle une référence n'est plus suivie (garde contre un cycle).
const PROFONDEUR_MAX: usize = 8;

/// L'en-tête d'un fichier de référence ; sa première ligne.
const EN_TETE: &str = "# empreinte du rendu d'un PDF — kesh-qrbill, format 1";

/// L'empreinte d'un PDF : tout ce qui se dessine, et rien d'autre.
#[derive(Debug, Clone, PartialEq)]
pub struct Empreinte {
    /// Le nombre de pages.
    pub pages: usize,
    /// La `MediaBox` de chaque page, une par ligne.
    pub media_box: String,
    /// Le condensat SHA-256 (hexadécimal) des opérations de contenu, pages à la suite.
    pub condensat_operations: String,
    /// Le nombre d'opérations de contenu.
    pub nombre_operations: usize,
    /// Le dictionnaire complet de chaque police, une par ligne.
    pub polices: String,
    /// Le dictionnaire des ressources de chaque page, références suivies, une par ligne.
    pub ressources: String,
    /// Les textes dessinés, `(x, y)` en mm depuis le bas à gauche.
    pub textes: Vec<(f32, f32, String)>,
}

/// Les opérations de contenu d'un PDF, pages à la suite.
pub fn operations(bytes: &[u8]) -> Vec<Operation> {
    let doc = document(bytes);
    doc.get_pages()
        .values()
        .flat_map(|page| {
            let brut = doc.get_page_content(*page).expect("contenu de page");
            Content::decode(&brut)
                .expect("contenu décodable")
                .operations
        })
        .collect()
}

/// Les textes dessinés d'un PDF, en mm depuis le bas à gauche : pour chaque `Tj`, son texte
/// décodé en WinAnsi, à la position du `Td` précédent.
///
/// ⚠️ `Document::extract_text` n'est pas employé : il lit en StandardEncoding et rend `Shne`
/// pour `Söhne` (Story 33-12a).
pub fn textes(bytes: &[u8]) -> Vec<(f32, f32, String)> {
    let mut textes = Vec::new();
    let mut position = (0.0, 0.0);
    for op in operations(bytes) {
        match op.operator.as_str() {
            "Td" => {
                position = (
                    nombre(&op.operands[0]) * MM_PAR_POINT,
                    nombre(&op.operands[1]) * MM_PAR_POINT,
                );
            }
            "Tj" => textes.push((position.0, position.1, texte_du_tj(&op))),
            _ => {}
        }
    }
    textes
}

/// Le texte d'un `Tj`, décodé en WinAnsi.
pub fn texte_du_tj(op: &Operation) -> String {
    let Object::String(octets, _) = &op.operands[0] else {
        panic!("opérande de Tj inattendue : {:?}", op.operands);
    };
    Document::decode_text(Some("WinAnsiEncoding"), octets)
}

/// Une opérande numérique de contenu PDF.
pub fn nombre(o: &Object) -> f32 {
    match o {
        Object::Integer(i) => *i as f32,
        Object::Real(r) => *r,
        autre => panic!("opérande numérique attendue : {autre:?}"),
    }
}

/// Le dictionnaire complet de chaque police de chaque page, références suivies, une ligne
/// par police (`<page> <nom de ressource> <dictionnaire>`).
pub fn polices(bytes: &[u8]) -> String {
    let doc = document(bytes);
    let mut sortie = String::new();
    for (numero, page) in doc.get_pages() {
        let Some(ressources) = ressources_de_page(&doc, page) else {
            continue;
        };
        let Ok(polices) = ressources.get(b"Font") else {
            continue;
        };
        let polices = suivre(&doc, polices);
        let Ok(polices) = polices.as_dict() else {
            panic!("/Font n'est pas un dictionnaire : {polices:?}");
        };
        for (nom, police) in polices.iter() {
            let _ = writeln!(
                sortie,
                "{numero} /{} {}",
                nom_lisible(nom),
                canonique(&doc, police, 0)
            );
        }
    }
    sortie
}

/// Le dictionnaire des ressources de chaque page, références suivies, une ligne par page.
pub fn ressources(bytes: &[u8]) -> String {
    let doc = document(bytes);
    let mut sortie = String::new();
    for (numero, page) in doc.get_pages() {
        let rendu = match ressources_de_page(&doc, page) {
            Some(r) => canonique(&doc, &Object::Dictionary(r.clone()), 0),
            None => "(aucune)".to_string(),
        };
        let _ = writeln!(sortie, "{numero} {rendu}");
    }
    sortie
}

/// La `MediaBox` de chaque page (héritée de l'arbre des pages au besoin), une par ligne.
pub fn media_box(bytes: &[u8]) -> String {
    let doc = document(bytes);
    let mut sortie = String::new();
    for (numero, page) in doc.get_pages() {
        let rendu = match attribut_herite(&doc, page, b"MediaBox") {
            Some(o) => canonique(&doc, &o, 0),
            None => "(absente)".to_string(),
        };
        let _ = writeln!(sortie, "{numero} {rendu}");
    }
    sortie
}

/// Le nombre de pages.
pub fn nombre_de_pages(bytes: &[u8]) -> usize {
    document(bytes).get_pages().len()
}

/// L'empreinte d'un PDF.
pub fn empreinte(bytes: &[u8]) -> Empreinte {
    let operations = operations(bytes);
    let doc = document(bytes);
    let mut condensat = Sha256::new();
    for op in &operations {
        condensat.update(operation_canonique(&doc, op).as_bytes());
        condensat.update(b"\n");
    }
    Empreinte {
        pages: nombre_de_pages(bytes),
        media_box: media_box(bytes),
        condensat_operations: format!("{:x}", condensat.finalize()),
        nombre_operations: operations.len(),
        polices: polices(bytes),
        ressources: ressources(bytes),
        textes: textes(bytes),
    }
}

impl Empreinte {
    /// L'écriture en texte de l'empreinte, diffable, relue par [`Empreinte::relire`].
    pub fn ecrire(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "{EN_TETE}");
        let _ = writeln!(s, "pages\t{}", self.pages);
        for l in self.media_box.lines() {
            let _ = writeln!(s, "media-box\t{}", echapper(l));
        }
        let _ = writeln!(s, "condensat-operations\t{}", self.condensat_operations);
        let _ = writeln!(s, "nombre-operations\t{}", self.nombre_operations);
        for l in self.polices.lines() {
            let _ = writeln!(s, "police\t{}", echapper(l));
        }
        for l in self.ressources.lines() {
            let _ = writeln!(s, "ressources\t{}", echapper(l));
        }
        for (x, y, t) in &self.textes {
            let _ = writeln!(s, "texte\t{x}\t{y}\t{}", echapper(t));
        }
        s
    }

    /// Relit une empreinte écrite par [`Empreinte::ecrire`].
    pub fn relire(texte: &str) -> Result<Empreinte, String> {
        let mut lignes = texte.lines();
        if lignes.next() != Some(EN_TETE) {
            return Err(format!("en-tête attendu : {EN_TETE}"));
        }
        let mut e = Empreinte {
            pages: 0,
            media_box: String::new(),
            condensat_operations: String::new(),
            nombre_operations: 0,
            polices: String::new(),
            ressources: String::new(),
            textes: Vec::new(),
        };
        for (i, ligne) in lignes.enumerate() {
            let erreur = |quoi: &str| format!("ligne {} : {quoi} : {ligne:?}", i + 2);
            let (cle, valeur) = ligne
                .split_once('\t')
                .ok_or_else(|| erreur("sans tabulation"))?;
            match cle {
                "pages" => e.pages = valeur.parse().map_err(|_| erreur("nombre"))?,
                "media-box" => pousser_ligne(&mut e.media_box, &desechapper(valeur)),
                "condensat-operations" => e.condensat_operations = valeur.to_string(),
                "nombre-operations" => {
                    e.nombre_operations = valeur.parse().map_err(|_| erreur("nombre"))?
                }
                "police" => pousser_ligne(&mut e.polices, &desechapper(valeur)),
                "ressources" => pousser_ligne(&mut e.ressources, &desechapper(valeur)),
                "texte" => {
                    let mut champs = valeur.splitn(3, '\t');
                    let x = champs.next().and_then(|v| v.parse().ok());
                    let y = champs.next().and_then(|v| v.parse().ok());
                    let t = champs.next();
                    match (x, y, t) {
                        (Some(x), Some(y), Some(t)) => e.textes.push((x, y, desechapper(t))),
                        _ => return Err(erreur("texte mal formé")),
                    }
                }
                _ => return Err(erreur("clé inconnue")),
            }
        }
        Ok(e)
    }

    /// Les écarts de `obtenue` à `self` (la référence), lisibles ; vide si elles sont égales.
    pub fn ecarts(&self, obtenue: &Empreinte) -> Vec<String> {
        let mut ecarts = Vec::new();
        if self.pages != obtenue.pages {
            ecarts.push(format!("pages : {} ≠ {}", self.pages, obtenue.pages));
        }
        if self.media_box != obtenue.media_box {
            ecarts.push(format!(
                "MediaBox : {:?} ≠ {:?}",
                self.media_box, obtenue.media_box
            ));
        }
        if self.condensat_operations != obtenue.condensat_operations {
            ecarts.push(format!(
                "condensat des opérations : {} ≠ {} ({} ≠ {} opérations)",
                self.condensat_operations,
                obtenue.condensat_operations,
                self.nombre_operations,
                obtenue.nombre_operations
            ));
        } else if self.nombre_operations != obtenue.nombre_operations {
            ecarts.push(format!(
                "nombre d'opérations : {} ≠ {}",
                self.nombre_operations, obtenue.nombre_operations
            ));
        }
        if self.polices != obtenue.polices {
            ecarts.push(format!(
                "polices :\n  attendu {:?}\n  obtenu  {:?}",
                self.polices, obtenue.polices
            ));
        }
        if self.ressources != obtenue.ressources {
            ecarts.push(format!(
                "ressources :\n  attendu {:?}\n  obtenu  {:?}",
                self.ressources, obtenue.ressources
            ));
        }
        if self.textes != obtenue.textes {
            let manquants: Vec<_> = self
                .textes
                .iter()
                .filter(|t| !obtenue.textes.contains(t))
                .take(20)
                .collect();
            let en_plus: Vec<_> = obtenue
                .textes
                .iter()
                .filter(|t| !self.textes.contains(t))
                .take(20)
                .collect();
            ecarts.push(format!(
                "textes ({} attendus, {} obtenus) — absents : {manquants:?} — en plus : {en_plus:?}",
                self.textes.len(),
                obtenue.textes.len()
            ));
        }
        ecarts
    }
}

/// Le mode des tests de capture, lu dans la variable d'environnement `KESH_REFERENCES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeReferences {
    /// `KESH_REFERENCES=ecrire` : écrit les références ; refuse d'en écraser une.
    Ecrire,
    /// `KESH_REFERENCES=comparer`, ou variable absente : relit les références et compare.
    Comparer,
}

/// Le mode des tests de capture. Variable absente : [`ModeReferences::Comparer`] — jamais
/// d'écriture involontaire. Toute autre valeur que `ecrire` ou `comparer` fait échouer.
pub fn mode_references() -> ModeReferences {
    match std::env::var("KESH_REFERENCES").as_deref() {
        Ok("ecrire") => ModeReferences::Ecrire,
        Ok("comparer") | Err(std::env::VarError::NotPresent) => ModeReferences::Comparer,
        autre => panic!("KESH_REFERENCES : `ecrire` ou `comparer` attendu, lu {autre:?}"),
    }
}

/// Écrit `contenu` dans `chemin`, en créant le dossier ; **échoue si le fichier existe**.
pub fn ecrire_sans_ecraser(chemin: &Path, contenu: &str) {
    use std::io::Write as _;
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier).expect("dossier des références");
    }
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(chemin)
        .unwrap_or_else(|e| panic!("référence {} : {e} (jamais écrasée)", chemin.display()));
    f.write_all(contenu.as_bytes())
        .expect("écriture de la référence");
}

/// Selon `mode`, écrit l'empreinte de référence `chemin` (sans écraser) ou la relit et rend
/// ses écarts à `obtenue` (vide : égales).
pub fn ecrire_ou_comparer(mode: ModeReferences, chemin: &Path, obtenue: &Empreinte) -> Vec<String> {
    match mode {
        ModeReferences::Ecrire => {
            ecrire_sans_ecraser(chemin, &obtenue.ecrire());
            Vec::new()
        }
        ModeReferences::Comparer => {
            let texte = std::fs::read_to_string(chemin)
                .unwrap_or_else(|e| panic!("référence {} : {e}", chemin.display()));
            let attendue = Empreinte::relire(&texte)
                .unwrap_or_else(|e| panic!("référence {} : {e}", chemin.display()));
            attendue.ecarts(obtenue)
        }
    }
}

/// Le SHA-256 de chacun des fichiers `chemins` (relatifs à `racine`), au format de
/// `sha256sum` (`<condensat>  <chemin>`), pour le `sources.sha256` d'un dossier de
/// références : il dit, après coup, quels fichiers de rendu la capture a compilés.
pub fn sources_sha256(racine: &Path, chemins: &[&str]) -> String {
    let mut s = String::new();
    for chemin in chemins {
        let octets =
            std::fs::read(racine.join(chemin)).unwrap_or_else(|e| panic!("source {chemin} : {e}"));
        let _ = writeln!(s, "{:x}  {chemin}", Sha256::digest(&octets));
    }
    s
}

// ----- interne -----

fn document(bytes: &[u8]) -> Document {
    Document::load_mem(bytes).expect("PDF relisible")
}

/// Suit une référence (une seule fois) ; rend l'objet tel quel sinon.
fn suivre<'a>(doc: &'a Document, o: &'a Object) -> &'a Object {
    doc.dereference(o).map(|(_, o)| o).unwrap_or(o)
}

/// Le dictionnaire des ressources d'une page (hérité de l'arbre au besoin).
fn ressources_de_page(doc: &Document, page: printpdf::lopdf::ObjectId) -> Option<Dictionary> {
    let o = attribut_herite(doc, page, b"Resources")?;
    suivre(doc, &o).as_dict().ok().cloned()
}

/// Un attribut de page, cherché sur la page puis sur ses parents (`/Parent`).
fn attribut_herite(doc: &Document, page: printpdf::lopdf::ObjectId, cle: &[u8]) -> Option<Object> {
    let mut noeud = doc.get_dictionary(page).ok()?;
    for _ in 0..PROFONDEUR_MAX {
        if let Ok(o) = noeud.get(cle) {
            return Some(o.clone());
        }
        let parent = noeud.get(b"Parent").ok()?.as_reference().ok()?;
        noeud = doc.get_dictionary(parent).ok()?;
    }
    None
}

/// Une opération de contenu en texte canonique : opérateur, puis opérandes.
fn operation_canonique(doc: &Document, op: &Operation) -> String {
    let mut s = op.operator.clone();
    for o in &op.operands {
        s.push(' ');
        s.push_str(&canonique(doc, o, 0));
    }
    s
}

/// Un objet PDF en texte canonique, références suivies jusqu'à [`PROFONDEUR_MAX`] (les
/// numéros d'objets n'y figurent pas : ils ne se dessinent pas). Les réels s'écrivent par
/// la représentation exacte de Rust (`{:?}` d'un `f32`) : un écart d'un ulp se voit.
fn canonique(doc: &Document, o: &Object, profondeur: usize) -> String {
    match o {
        Object::Null => "null".into(),
        Object::Boolean(b) => b.to_string(),
        Object::Integer(i) => i.to_string(),
        Object::Real(r) => format!("{r:?}"),
        Object::Name(n) => format!("/{}", nom_lisible(n)),
        Object::String(octets, format) => {
            let hex: String = octets.iter().map(|b| format!("{b:02X}")).collect();
            match format {
                StringFormat::Literal => format!("({hex})"),
                StringFormat::Hexadecimal => format!("<{hex}>"),
            }
        }
        Object::Array(a) => {
            let elements: Vec<String> = a
                .iter()
                .map(|e| canonique(doc, e, profondeur + 1))
                .collect();
            format!("[{}]", elements.join(" "))
        }
        Object::Dictionary(d) => dictionnaire_canonique(doc, d, profondeur),
        Object::Stream(flux) => {
            let contenu = flux
                .decompressed_content()
                .unwrap_or_else(|_| flux.content.clone());
            format!(
                "stream{}sha256:{:x}",
                dictionnaire_canonique(doc, &flux.dict, profondeur),
                Sha256::digest(&contenu)
            )
        }
        Object::Reference(id) => {
            if profondeur >= PROFONDEUR_MAX {
                return "R".into();
            }
            match doc.get_object(*id) {
                Ok(cible) => canonique(doc, cible, profondeur + 1),
                Err(_) => "R?".into(),
            }
        }
    }
}

/// Un dictionnaire en texte canonique, clés **triées** (l'ordre d'écriture ne se dessine
/// pas). `/Parent` n'est pas suivi (il remonterait l'arbre des pages).
fn dictionnaire_canonique(doc: &Document, d: &Dictionary, profondeur: usize) -> String {
    let mut paires: Vec<(String, String)> = d
        .iter()
        .map(|(cle, valeur)| {
            let valeur = if cle.as_slice() == b"Parent" {
                "R".to_string()
            } else {
                canonique(doc, valeur, profondeur + 1)
            };
            (nom_lisible(cle), valeur)
        })
        .collect();
    paires.sort();
    let corps: Vec<String> = paires
        .into_iter()
        .map(|(cle, valeur)| format!("/{cle} {valeur}"))
        .collect();
    format!("<<{}>>", corps.join(" "))
}

fn nom_lisible(n: &[u8]) -> String {
    String::from_utf8_lossy(n).into_owned()
}

fn pousser_ligne(cible: &mut String, ligne: &str) {
    cible.push_str(ligne);
    cible.push('\n');
}

/// Échappe `\`, tabulation, retour chariot et saut de ligne (une valeur par ligne).
fn echapper(s: &str) -> String {
    let mut sortie = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => sortie.push_str("\\\\"),
            '\t' => sortie.push_str("\\t"),
            '\n' => sortie.push_str("\\n"),
            '\r' => sortie.push_str("\\r"),
            c => sortie.push(c),
        }
    }
    sortie
}

fn desechapper(s: &str) -> String {
    let mut sortie = String::with_capacity(s.len());
    let mut car = s.chars();
    while let Some(c) = car.next() {
        if c != '\\' {
            sortie.push(c);
            continue;
        }
        match car.next() {
            Some('t') => sortie.push('\t'),
            Some('n') => sortie.push('\n'),
            Some('r') => sortie.push('\r'),
            Some(autre) => sortie.push(autre),
            None => sortie.push('\\'),
        }
    }
    sortie
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_echappement_fait_l_aller_retour() {
        for s in ["", "a\tb", "a\\nb", "\\", "x\ny\r\n", "é « » €"] {
            assert_eq!(desechapper(&echapper(s)), s, "{s:?}");
        }
    }

    #[test]
    fn une_empreinte_ecrite_se_relit_a_l_identique() {
        let e = Empreinte {
            pages: 2,
            media_box: "1 [0 0 595.2756 841.8898]\n2 [0 0 1.5 2]\n".into(),
            condensat_operations: "ab12".into(),
            nombre_operations: 42,
            polices: "1 /F0 <</BaseFont /Helvetica>>\n".into(),
            ressources: "1 <<>>\n2 (aucune)\n".into(),
            textes: vec![
                (20.0, 159.0, "Conseil\tà\\n".into()),
                (0.1, -3.25, String::new()),
            ],
        };
        assert_eq!(Empreinte::relire(&e.ecrire()), Ok(e));
    }
}
