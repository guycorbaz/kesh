# Story 44-1b : QR-facture — les paliers intermédiaires de la cascade (R2, R3, P1, P2) (#1230)

Status: backlog

*Issue de la story : **#1230** (R2, R3, P1, P2) ; #944 se ferme avec la 44-1a. Fiche courte écrite le 2026-11-17 au découpage de la 36-12b (validation P1, H1, option (b)). **Release ultérieure**
(0.20.0 ou la suivante, selon le plan du Project Lead) : elle ne se tient pas dans la 0.19.0 et n'entre pas dans la
coupure du 2026-11-19. Elle **part de la 44-1a fusionnée** (`44-1a-qr-facture-grille.md`), même fichier `pdf.rs`.
À rattacher à un epic de sa release au kickoff (la règle « un epic = une release » interdit de la laisser dans l'Epic 44
une fois celui-ci clos) ; **à valider (P1) avant tout développement**. Aucune migration, aucune ligne de `kesh-api`,
`kesh-db`, `kesh-i18n`.*

## Story

En tant que titulaire, je veux qu'une adresse trop longue pour le récépissé ou la colonne d'information **perde le moins
possible** avant d'être raccourcie — des tailles plus petites, puis des noms sur deux lignes, le message raccourci par lignes —
plutôt que de sauter d'un coup de la rue omise (R1, livrée par la 44-1a) ou de l'adresse complète au dernier palier de
secours de la 44-1a.

## Ce que la 44-1a lui laisse (à relire à l'étape 0)

- la mise en page **pure à paliers** (chacun avec son critère « tient ») : le nombre de paliers est ouvert, la 44-1b en
  **insère** entre R1 et R4, entre P0 et P3, sans toucher à R0, R1, R4, P0, P3 ; **R1** (rue et numéro omis au récépissé,
  créancier et débiteur ensemble, sans objet en type K ; R0 tient jusqu'à 8 lignes d'adresse avec une référence QR, 11 sans)
  est livré par la 44-1a, avec sa fixture et sa mutation ;
- les constantes de pas et de grille (D2 de la 44-1a), `couper_ligne` à la largeur avec « ... », `largeur_mm` ;
- le test `aucune_entree_admise_n_echoue` (propriété), le relevé indépendant `pire_cas_boites_independantes`, le dispositif de
  mutations ; les tests de position déjà mis à jour.

## Périmètre (étapes d'origine, D5 de la 36-12b, à préciser avant la validation)

- **R2** (après R1) : valeurs **et intitulés** à 6 pt, pas de ligne
  3,5 × 6 / 8 (§ 3.6.2 ; écart à § 3.4, intitulés deux points sous les valeurs, assumé) ; **R3** : noms ramenés à **deux
  lignes** avec « ... » (**tous** les noms ou ceux qui dépassent : à trancher) ;
- **P1** : message raccourci aux lignes qui tiennent (**nombre de lignes à fixer**, une au moins), « ... » en fin ; **P2** : noms
  à deux lignes, « ... ».
- À chaque palier : son critère « tient » écrit en nombre (bas ≥ 41,52 au récépissé, ≥ 5 en colonne), **une fixture qui le
  déclenche et seulement lui**, **une mutation** qui le retire et un test qui rougit ; l'AC6 de la 44-1a (hauteur de R4 et P3
  par construction) reste vraie.
- Manuel : décrire les paliers ajoutés (taille réduite, noms sur deux lignes, message raccourci par lignes) dans le
  paragraphe de la 44-1a, qui dit déjà la rue omise.
- Note de version : une ligne, rubrique *Modifié*.

## Limites à lire

Les écarts D8 de la 44-1a (nom du créancier tronqué, rue supprimée par R4) **diminuent** avec cette story : ils ne disparaissent
qu'aux adresses qui dépassent encore tous les paliers intermédiaires. (R1, réaliste, est déjà dans la 44-1a.) Étendue : un seul module (`kesh-qrbill`) et le manuel ; si la validation la juge encore
trop large, la couper en récépissé (R2, R3) et colonne (P1, P2).

## Dev Agent Record

*(à remplir)*

## Change Log

- 2026-11-17 — Fiche écrite au découpage de la 36-12b (remédiation de la validation P1, H1 (b)). Validation non lancée.
- 2026-11-17 — **R1 retiré de cette fiche** (remédiation de la validation P2 de la 44-1a, M1 : il couvre les adresses
  réalistes et passe dans la 44-1a) ; restent R2, R3, P1, P2.
