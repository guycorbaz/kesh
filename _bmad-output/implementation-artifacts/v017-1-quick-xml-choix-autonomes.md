# Release 0.17.0 — registre des choix autonomes de la Story v017-1 (quick-xml)

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** au registre de la release.
Fiche : `v017-1-quick-xml.md`. Numérotation `C-v017-1-k`. Issue : #789 (P1), en `refs` seulement. Les numéros de ligne
cités sont ceux de `c5101b07`. Chaque choix : contexte, option retenue, alternatives écartées, réversibilité.

---

### C-v017-1-1 — Version cible : `quick-xml` 0.42, et non 0.41

- **Contexte** : les deux avis sont corrigés dès 0.41.0 (`patched = [">= 0.41.0"]`). 0.41 ne change presque rien à
  l'API utilisée par Kesh (seule la dépréciation de `unescape_value`, venue de 0.40). 0.42.0 (2026-08-22) refond les
  types de nom et de texte en `&str` (#963), mais ferme en plus deux autres chemins de déni de service sur le même
  lecteur : le débordement de profondeur au-delà de `u16::MAX` (#977, `TooDeeplyNested`) et un plafond **global** des
  liaisons en portée (#980), là où 0.41 ne borne que par élément (un attaquant peut répartir 256 déclarations sur chaque
  niveau d'imbrication).
- **Retenu** : **0.42**, avec AC7 pour l'imbrication profonde. L'adaptation porte sur une dizaine de lignes de
  `camt053/mod.rs` ; elle est encadrée par la caractérisation AC9, qui interdit tout changement de sortie.
- **Écartées** : (a) 0.41 — corrige les deux avis au moindre coût, mais laisse #977 ouvert et appelle une seconde montée
  à courte échéance ; (b) rester en 0.39 et contourner (`with_checks(false)` dans `attr_string`) — ne couvre pas
  RUSTSEC-2026-0195, pour lequel l'avis dit qu'il n'existe aucun contournement propre.
- **Réversible** : oui — revenir à `"0.41"` ne demande que de défaire l'adaptation #963 ; AC9 resterait le juge.
- **Nuance (validation P1, F-7, 2026-10-30)** : #977 n'est un déni de service qu'en debug — 0.39.4 `src/name.rs:668`
  `self.nesting_level += 1;` sur un `u16` panique sous `overflow-checks` et déborde **en silence** en release (résolution
  d'espace de noms faussée, sans arrêt). L'argument décisif pour 0.42 reste le plafond global (#980) et une seule montée ;
  le risque que 0.42 ajoute sur des données gardées est #963 (types `&str`, validation UTF-8 de toute l'entrée), dont la
  seule garantie est la caractérisation AC9 et dont la seule divergence admise est C-v017-1-11.

### C-v017-1-2 — `kesh-payment` monte aussi, bien qu'aucun avis ne l'atteigne

- **Contexte** : pain.001 n'écrit que du XML ; les avis portent sur la lecture. Laisser `kesh-payment` en 0.39 garderait
  deux versions de `quick-xml` dans l'arbre, et `cargo audit` continuerait de signaler la 0.39.4 (il juge le lock, non
  l'usage).
- **Retenu** : monter les deux crates ensemble ; une seule entrée `quick-xml` au lock (AC1) ; pain.001 tenu octet pour
  octet par AC12.
- **Écartées** : monter `kesh-import` seul — `cargo audit` resterait rouge, AC2 inatteignable.
- **Réversible** : oui.

### C-v017-1-3 — Caractérisation par références JSON produites sous 0.39.4, messages de `quick-xml` exclus

- **Contexte** : la contrainte du 2026-10-30 exige qu'un relevé déjà importé reste lisible et inchangé. La seule voie
  par laquelle la montée peut altérer la détection de doublons est la sortie du parseur (la `DuplicateKey` de ligne en
  dérive ; le hash de fichier, non). Les messages d'erreur de `quick-xml` changent d'une version à l'autre et ne sont ni
  stockés ni montrés tels quels à l'écran (`BANK_IMPORT_MALFORMED_XML` et sa clé i18n).
- **Retenu** : références `camt053_golden/*.json` de la sortie entière (`serde_json`) pour tout fichier CAMT du dépôt et quatre
  fixtures neuves, produites **avant** la montée dans un commit sans code ; les erreurs comparées par variant, et par
  message seulement quand Kesh le rédige.
- **Écartées** : (a) relire les tests existants seulement — ils assertent quelques champs, pas la sortie entière, et
  aucune fixture ne porte BOM, CRLF intérieur, CDATA, entités ni Latin-1 ; (b) comparer aussi les messages de
  `quick-xml` — rougirait sans changement de donnée ; (c) `insta` — dépendance de plus pour quatre lignes de comparaison.
- **Réversible** : oui (tests seulement).

### C-v017-1-4 — Tests hostiles bornés par un délai dans un fil, tailles choisies sous le plafond d'upload

- **Contexte** : il faut des tests rouges sous 0.39 et verts sous 0.42, sans réseau et sans figer la suite. Sous 0.39,
  le test d'attributs ne se termine pas en un temps raisonnable ; un test qui pend bloque nextest.
- **Retenu** : parse dans `std::thread::spawn`, attente `recv_timeout(10 s)` ; 200 000 attributs (≈ 2,4 Mo < 10 MiB par
  défaut), 10 000 déclarations, 70 000 niveaux. Le délai est la seule assertion d'AC5 ; AC6 asserte le refus ; AC7
  le refus sans panique (`Timeout` et `Disconnected` distingués — remédiation P1, R-3).
- **Écartées** : (a) mesurer le temps et comparer à un seuil serré — instable entre poste et CI ; (b) mesurer
  l'allocation (allocateur compteur) — infrastructure lourde pour un comportement que le refus borne déjà ; (c) tests
  `#[ignore]` — un test qui ne tourne pas se tait.
- **Réversible** : oui (seuils ajustables, écrits en constantes nommées).

### C-v017-1-5 — Plafonds de `quick-xml` laissés à leurs défauts

- **Contexte** : 0.42 borne les liaisons en portée à 128 par défaut, réglable. Un relevé CAMT.053 réel en porte deux ou
  trois (`xmlns`, `xmlns:xsi`).
- **Retenu** : aucun réglage ; AC6 (c) prouve qu'un fichier à cinq déclarations passe. M2 vérifie qu'un relèvement du
  plafond se verrait.
- **Écartées** : (a) abaisser le plafond (par exemple à 16) — gain marginal, risque de refuser un fichier bancaire
  atypique ; (b) le relever — sans objet.
- **Réversible** : oui.

### C-v017-1-6 — Signatures publiques `v04::parse` / `v08::parse` gardées

- **Contexte** : elles exposent `quick_xml::NsReader` dans l'API publique de `kesh-import` ; aucun appelant hors du crate ;
  crate non publié.
- **Retenu** : les garder telles quelles (périmètre minimal). Le changement de type exposé est couvert par le passage à
  0.17.0 (mineur en 0.x) que porte la v016-2.
- **Écartées** : les passer en `pub(crate)` pour sortir `quick-xml` de l'API — défendable, mais c'est une décision
  d'architecture du crate publiable (décision archi #7), hors de l'objet de l'issue. À proposer en issue si l'on veut
  découpler.
- **Réversible** : oui.

### C-v017-1-7 — Le défaut des références d'entité reste hors périmètre, figé par la caractérisation

- **Contexte** : `quick-xml` (depuis 0.38) émet `&amp;`, `&#xE9;`… en `Event::GeneralRef`, que le parseur ignore
  (`mod.rs:246`) ; le dernier fragment de texte écrase les précédents dans les champs affectés par `= Some(txt)`
  (`mod.rs:473` pour `Nm`). Un « Dupont & Fils » devient « Fils ». Le corriger changerait la sortie du parseur — et la
  `DuplicateKey` d'une ligne dont la référence porte une entité —, ce que la contrainte du 2026-10-30 interdit ici.
- **Retenu** : figer le comportement actuel dans `v04_characterization.xml` avec un commentaire qui renvoie à l'issue ;
  **tracé par #819** (bug, P2 : contrepartie et libellés faux sans effet sur les montants ; la correction doit dire ce
  qu'il advient de la détection de doublons de ligne pour les relevés déjà importés).
- **Écartées** : corriger ici — non mécanique, change des données dérivées.
- **Réversible** : oui.

### C-v017-1-8 — Le parse sur l'exécuteur Tokio reste tel quel

- **Contexte** : `parse_camt053` est appelé dans le gestionnaire async (`routes/bank_imports.rs:1045`, `:1249`), sans
  `spawn_blocking`. Même linéaire, un parse de 100 MiB occupe un fil de l'exécuteur.
- **Retenu** : hors périmètre ; **tracé par #820** (P3, défense en profondeur : `spawn_blocking` pour CAMT et CSV).
- **Écartées** : l'intégrer — touche deux gestionnaires et les chemins d'erreur, non mécanique.
- **Réversible** : —

### C-v017-1-9 — `\r` dans un texte pain.001 : écart assumé

- **Contexte** : 0.42 échappe `\r` en `&#13;` dans `BytesText::new` (#990 ; établi dans la source 0.42.0 :
  `src/escape.rs:109` ajoute `b'\r'` à l'ensemble échappé, `src/events/mod.rs:586-587` `BytesText::new` =
  `from_escaped(escape(content))` ; 0.39.4 `src/escape.rs:106` n'échappait que `< > & ' "`). Sous 0.39, un `\r` sortait brut — qu'un
  lecteur XML normalise en `\n`. L'écriture 0.42 est la plus fidèle. Le fichier pain.001 est régénéré à la demande, non
  stocké.
- **Retenu** : la référence d'AC12 n'inclut pas de `\r` ; l'écart est écrit dans le commentaire du test.
- **Écartées** : filtrer `\r` en amont — changement de comportement hors objet.
- **Réversible** : oui.

### C-v017-1-10 — Pas de contrôle sur relevé réel par l'agent

- **Contexte** : la caractérisation porte sur des fixtures synthétiques ; un fichier bancaire réel pourrait porter une
  forme qu'elles n'ont pas. Les relevés de Guy sont dans `~/kesh-test/`, qu'aucun agent n'ouvre.
- **Retenu** : contrôle **facultatif**, par l'orchestrateur ou Guy, en local, rapporté « conforme / non conforme » sans
  chiffre, condensé de sortie écrit dans `$GATE_LOGS` seulement.
- **Écartées** : un test `#[ignore]` lisant un chemin en variable d'environnement (modèle `ubs_fichier_reel.rs`) —
  envisageable, mais comparer avant/après exige deux binaires ; le condensé suffit.
- **Réversible** : oui.

### C-v017-1-11 — Un fichier non UTF-8 hors du texte lu est désormais refusé : divergence admise

*(Ajouté le 2026-10-30, remédiation de la validation P1 — F-1 et R-5 ; décision de l'orchestrateur.)*

- **Contexte** : sous 0.39.4, seul le texte des éléments est décodé (`camt053/mod.rs:235`, `t.decode()` sur les
  `Event::Text`) ; un octet non UTF-8 dans un commentaire, une section CDATA, une instruction ou une valeur d'attribut non
  lue passait, et le relevé s'importait. 0.42.0 valide l'UTF-8 de chaque section lue (`src/reader/buffered_reader.rs:297`
  pour les sections `<!…>`, `:240`, `:588`) et refuse le fichier entier. Le parse précédant la recherche du hash
  (`routes/bank_imports.rs:1036-1045`), un tel fichier déjà importé en 0.16 serait, réimporté, refusé au lieu d'être
  signalé en doublon. Les données en base ne sont pas touchées.
- **Retenu** : divergence **admise** — la norme ISO 20022 impose l'UTF-8 aux messages XML ; un fichier qui ne l'est pas
  n'est pas un relevé conforme. Elle est **prouvée et écrite** : fixture `v04_latin1_hors_texte.xml` (référence 0.39
  consignée sous `v04_latin1_hors_texte.0.39.json`), exception nominative dans AC9, refus
  `BANK_IMPORT_MALFORMED_XML` asserté par AC8, phrase au CHANGELOG (AC14), ligne au § *Ce que les gardes ne voient pas*.
  Rattaché : la normalisation des valeurs d'attribut (`normalized_value`, 0.42 `src/events/attributes.rs:75`, `:85`) ne
  change qu'un `Ccy` portant un blanc littéral, qui n'est pas un code de devise — écrit, non figé.
- **Écartées** : (a) prétraiter l'entrée pour retirer ou transcoder les octets hors texte — code de plus, sur un chemin
  d'entrée hostile, pour tolérer un fichier non conforme ; (b) activer la fonctionnalité `encoding` de `quick-xml`
  (`DecodingReader`) — change aussi la lecture des fichiers Latin-1 **dans** le texte, aujourd'hui refusés : élargissement
  hors objet.
- **Réversible** : oui (un prétraitement pourrait s'ajouter plus tard sans toucher aux données).
