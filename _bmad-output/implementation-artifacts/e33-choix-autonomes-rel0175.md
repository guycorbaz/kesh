# Release 0.17.5 — registre des choix autonomes de l'intégration (clé rel0175)

Tenu le 2026-11-07 par l'agent d'intégration (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-rel0175`, branche
`release/0.17.5` depuis `origin/main` = `d9b53d38`. Journaux : `kesh-gate-logs/rel0175/`.

- **C-rel0175-1 — `first_qr_refused_char` (33-5) suit le jeu de la 33-12a.** *Contexte* : conflit dans
  `crates/kesh-qrbill/src/validation.rs` ; la 33-5 ajoutait la fonction publique sur `is_six_annex_c_char`, que la
  33-12a remplace par `est_caractere_qr` (§ 4.1.1 de SIX). *Retenu* : la fonction de la 33-5 garde sa signature et
  délègue à `est_caractere_qr`, comme la fiche 33-12a le prévoyait (C-e3312-2) ; `is_six_annex_c_char` disparaît.
  *Conséquence* : la saisie de l'adresse de la société admet désormais « & », `<`, `>`, `"`, comme la QR-facture.
  *Écartée* : garder les deux règles, qui refuseraient à la saisie ce que le PDF accepte. *Réversible* : oui.
- **C-rel0175-2 — Renuméroter les choix en doublon, sans réécrire les Change Logs.** *Contexte* : six branches
  avaient numéroté à partir de `C-e334-32`, et la 33-3b avait repris `C-e332-25`/`26`. *Retenu* : la 33-4 garde
  32 à 39 ; 33-5 → 40, 41 ; 33-6 → 42, 43 ; 33-8 → 44 à 46 ; 33-7 → 47, 48 ; 33-10 → 49 à 54 ; 33-3b → C-e332-29, 30.
  L'ancien numéro est écrit à chaque entrée ; les quatre commentaires de code et le champ *Choix* de la fiche 33-3b
  suivent ; les Change Logs et prompts, relevés datés, gardent l'ancien numéro. *Réversible* : oui.
- **C-rel0175-3 — Registre des audits : 138 tracées sur 145 routes.** *Contexte* : 33-5 (une route tracée de plus) et
  33-7 (le mot de passe passe d'exemptée à tracée) amenaient chacune « 137 ». *Retenu* : 138 + 4 + 3 = 145, au test
  et au manuel, mesuré par le gate. *Réversible* : sans objet.
- **C-rel0175-4 — Gardes de décompte recomptées par elles-mêmes.** `sitesTotal` 2922 (rendu par la garde),
  `CANDIDATES_ATTENDUES` 85 et `conforme` 71 (verts au premier essai), jamais additionnés à la main.
- **C-rel0175-5 — Corriger à l'intégration le taux de TVA des tests de `dates_bornes_e2e`.** *Contexte* :
  `scripts/gate-ci.sh` rougit sur `c_tva_*` : `10 + seq()` dépasse 100 sous `cargo test`, un processus par binaire,
  ce que la CI aurait fait aussi. *Retenu* : un correctif de test seul (`19500f88`), puis gates complets rejoués.
  *Écartée* : renvoyer à une story, la release ne passant pas la CI sans lui. *Réversible* : oui.
- **C-rel0175-6 — Deux recettes de montée.** *Contexte* : le peuplement de `recette-montee.sh` ne porte qu'un exercice,
  et son retour passe par le dump, non par `pre-upgrade/`. *Retenu* : le script tel quel (crochet `--apres` pour la
  numérotation, `--diff-attendu migrations,supplier_invoices,fiscal_years,company_invoice_settings,api`), plus une
  recette complémentaire hors dépôt (`kesh-gate-logs/rel0175/recette-complement.sh`) : deux exercices, retour par
  `pre-upgrade/` dans une 0.17.4 neuve, refus d'un `.keshbackup` 0.17.5, refus de migrer sans sauvegarde,
  échappatoire. Son montage fait de l'exercice unique du jeu « with-company » (2020-2030, vide) un exercice 2025, par
  SQL sur la base jetable, avant toute écriture. *Réversible* : sans objet.
- **C-rel0175-7 — `closes` de la PR.** Quinze : #926, #922, #915 (reliquat tracé en #934), #918, #606, #755, #909,
  #450 (suite #951), #435 (fermée sur le mot de passe, décision de Guy), #729, #776, #486, #424, #925, #941 (suites
  #942 à #944, #955 à #957). `refs` pour #928 (AC-6 de la 33-11 à relever sur la PR) et #923 (fiches 2 et 3 ouvertes).
- **C-rel0175-8 — CHANGELOG `[0.17.5]` réordonné.** Treize sections réunies en *Ajouté* (4) et *Corrigé* (19), puces
  reprises mot pour mot, un avertissement de tête et un « En bref » pour le Project Lead. La réserve de l'entrée du
  filtre de CI (33-11) est laissée : elle tombe avec l'AC-6.
