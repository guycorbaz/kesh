//! Lecture du texte d'un PDF de rapport — **soutien de tests** (Story 46-3, AC-2
//! et AC-4).
//!
//! L'extracteur partagé des tests de `kesh-report` et de `kesh-api` : les tests
//! de rendu de ce crate et les tests d'intégration des exports relisent le PDF
//! par ces fonctions, sans copie. Il ne lit que ce qu'écrit [`crate::pdf`] —
//! des blocs `BT … <x> <y> Td <hex> Tj ET` de printpdf 0.7, **non compressés** :
//! ce qui n'est vrai que hors du profil `release` (sous `cargo test`). Aucun
//! `pdftotext` (non garanti en CI).
//!
//! Il n'est appelé par aucun code de production ; il est public parce qu'un
//! test d'intégration de `kesh-api` ne voit pas le `#[cfg(test)]` d'une
//! dépendance.

/// Un texte posé dans le PDF, dans l'ordre d'écriture.
#[derive(Debug, Clone, PartialEq)]
pub struct TextePose {
    /// Abscisse de départ, en points.
    pub x_pt: f32,
    /// Ordonnée de la ligne de base, en points depuis le bas de la page.
    pub y_pt: f32,
    /// Le texte, décodé octet par octet en Latin-1 (approximation de WinAnsi :
    /// l'ellipse `…`, `0x85` en WinAnsi, s'y lit `\u{85}`).
    pub texte: String,
}

/// Les textes d'un PDF non compressé, dans l'ordre d'écriture : chaque bloc
/// `BT … <x> <y> Td <hex> Tj` de printpdf.
pub fn poses(pdf: &[u8]) -> Vec<TextePose> {
    let s = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    for bloc in s.split("BT").skip(1) {
        let Some(td) = bloc.find(" Td") else { continue };
        let mut coord = bloc[..td].split_whitespace().rev();
        let (Some(y), Some(x)) = (coord.next(), coord.next()) else {
            continue;
        };
        let (Ok(y_pt), Ok(x_pt)) = (y.parse::<f32>(), x.parse::<f32>()) else {
            continue;
        };
        let Some(a) = bloc.find('<') else { continue };
        let Some(b) = bloc[a..].find("> Tj") else {
            continue;
        };
        let hex = &bloc[a + 1..a + b];
        let texte: String = (0..hex.len() / 2)
            .filter_map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok())
            .map(char::from)
            .collect();
        out.push(TextePose { x_pt, y_pt, texte });
    }
    out
}

/// Les **lignes** de texte d'un PDF non compressé : les textes consécutifs de
/// même ordonnée (à un centième de point près) forment une ligne, lus de gauche
/// à droite et séparés par une espace ; texte **normalisé** — espaces répétées
/// réduites, bords retirés, l'ellipse WinAnsi (`0x85`) rendue `…`. Le retrait
/// d'une ligne (son abscisse) ne s'y lit donc pas : [`poses`] le donne.
pub fn lignes(pdf: &[u8]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut courante: Vec<TextePose> = Vec::new();
    let vider = |courante: &mut Vec<TextePose>, out: &mut Vec<String>| {
        if courante.is_empty() {
            return;
        }
        courante.sort_by(|a, b| a.x_pt.total_cmp(&b.x_pt));
        let brut: Vec<&str> = courante.iter().map(|p| p.texte.as_str()).collect();
        out.push(normaliser(&brut.join(" ")));
        courante.clear();
    };
    for p in poses(pdf) {
        if courante
            .last()
            .is_some_and(|d| (d.y_pt - p.y_pt).abs() > 0.01)
        {
            vider(&mut courante, &mut out);
        }
        courante.push(p);
    }
    vider(&mut courante, &mut out);
    out
}

/// Normalise une ligne de texte : l'ellipse WinAnsi rendue `…`, espaces
/// répétées réduites à une, bords retirés.
pub fn normaliser(texte: &str) -> String {
    texte
        .replace('\u{85}', "…")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un flux factice au format de printpdf : deux textes sur une ligne (écrits
    /// de droite à gauche), puis un troisième plus bas.
    #[test]
    fn regroupe_par_ordonnee_et_lit_de_gauche_a_droite() {
        let flux = "BT /F1 10 Tf 500.0 700.0 Td <3132> Tj ET\n\
                    BT /F1 10 Tf 40.0 700.0 Td <41> Tj ET\n\
                    BT /F1 10 Tf 40.0 688.0 Td <422085> Tj ET";
        assert_eq!(lignes(flux.as_bytes()), vec!["A 12", "B …"]);
        let p = poses(flux.as_bytes());
        assert_eq!(p.len(), 3);
        assert_eq!((p[1].x_pt, p[1].y_pt), (40.0, 700.0));
    }
}
