# Registre des choix autonomes — Story 49-1b (clé `e491b`)

*Agent de développement de la 49-1b (Opus 5.5), 2026-11-19. Release 0.20.0, #1242. Chaque choix : contexte, option
retenue, autres options, réversibilité.*

## C-e491b-1 — L'empreinte compare les opérations de contenu, non l'octet neutralisé du fichier

- **Contexte** : D4 laisse au développeur le choix, à l'étape 0, entre le condensat des opérations et l'octet du
  fichier neutralisé du `/ID`.
- **Retenu** : le condensat des opérations (et polices, ressources, `MediaBox`, pages, textes).
- **Motif** (lu dans printpdf 0.7) : l'aléa n'est pas que le `/ID` — `xmp_metadata.rs:42` tire aussi au hasard
  l'identifiant d'instance de la métadonnée XMP ; `pdf_document.rs` élague les objets (`optimize`) en `release`
  seulement, si bien que les octets d'un test (`debug`) et de la production diffèrent ; au niveau HTTP, la date de
  création est `now_utc()`. Neutraliser l'octet demanderait trois neutralisations et ne vaudrait que pour
  `kesh-qrbill` ; une seule méthode sert aux deux niveaux.
- **Autres** : octet neutralisé du `/ID` et de la XMP, au seul niveau `kesh-qrbill`.
- **Réversible** : oui (une autre méthode s'ajouterait au comparateur, avec de nouvelles références).

## C-e491b-2 — L'empreinte porte aussi les ressources de chaque page

- **Contexte** : D4 énumère condensat des opérations, polices, `MediaBox`, pages et textes.
- **Retenu** : en plus, le dictionnaire des ressources de chaque page, références suivies (fonction `ressources`).
  Une ressource neuve (état graphique, groupe de contenu) changerait le rendu sans forcément toucher une police.
- **Autres** : s'en tenir à l'énumération de D4.
- **Réversible** : oui ; posé dans le commit C, avant la capture.

## C-e491b-3 — Les scénarios de `kesh-qrbill` lisent les libellés par défaut, non les catalogues

- **Contexte** : les neuf scénarios de l'AC1 ont besoin de libellés.
- **Retenu** : `QrBillI18n::default()` (les libellés de repli de `types.rs`, fichier de `RENDU`), et pour l'avoir
  les deux surcharges de `credit_notes.rs` écrites en littéraux. Une autre story de la 0.20.0 qui changerait un
  libellé du catalogue ne fait pas rougir la preuve du **rendu** ; le chemin HTTP (AC2), lui, lit les catalogues
  et rougirait — à juste titre : le PDF des factures changerait.
- **Autres** : `QrBillI18n::depuis_ftl` du catalogue français.
- **Réversible** : oui, avec de nouvelles références.

## C-e491b-4 — T1, T2 et T3 entrent après le commit F

- **Contexte** : la fiche range T0, T0-bis et T1-bis dans le commit C.
- **Retenu** : T1 (`kesh-qrbill`), T2 et T3 (`kesh-api`) relisent les références ; posés dans C, ils échoueraient
  faute de fixtures. Ils appellent la même fonction de vérification que T0 et T0-bis, en mode `comparer`, ajoutés
  au même fichier après F.
- **Réversible** : sans objet.
