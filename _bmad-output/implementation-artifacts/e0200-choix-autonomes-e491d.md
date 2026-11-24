# Release 0.20.0 — choix autonomes de la Story 49-1d (clé `e491d`)

Registre du développement de `49-1d-types-de-lignes-recette-montee` (Opus 5.5, `bmad-dev-story`, reprise du
2026-11-24). Chaque choix : contexte, option retenue, autres options, réversibilité.

## C-e491d-1 — La version de départ est la 0.19.1

- **Contexte** : la branche d'épique a repris `main` (v0.19.1, publiée le 2026-11-21 sans migration, `87e00ba05`,
  fusionnée dans la story par `0d033d36e`). La fiche, rédigée le 2026-11-19, part de l'image 0.19.0 et du tag
  `773df80da`. La production tourne en 0.19.1 depuis le 2026-11-21.
- **Retenu** : partout où la fiche nomme la 0.19.0 comme **version de départ**, lire 0.19.1 : image de la sonde, de
  la recette (AC10) et du script complémentaire (`gcorbaz/kesh:0.19.1`, `sha256:50b01c7cf627`) ; nom de la
  sauvegarde `pre-upgrade/` (`-0.19.1-to-*`, et `-0.19.1-to-0.19.1` à la tête de la story, les crates étant en
  0.19.1) ; « Le retour à la 0.19.1 » au manuel ; Note de version ; base du contrôle de la perte assumée (tag
  `v0.19.1`, `fe84ff9ce`). Les sites opératoires de la fiche sont corrigés ; les mentions historiques (précédent
  `rel0190`, section « Passer à la 0.19.0 » du manuel, #1448, phrases de la section 0.19.0 reprises) restent.
  Prémisses relues au code : entre v0.19.0 et v0.19.1, `crates/kesh-qrbill/src` ne change que la visibilité de
  `largeur_mm` (aucun rendu), `crates/kesh-api/src/lib.rs`, `routes/issued_invoice_pdf.rs`,
  `routes/invoice_pdf_service.rs`, `routes/credit_notes.rs`, `admin_backup/`, `main.rs`, `pre_upgrade_backup.rs` et
  `frontend/src/lib/features/admin-restore/` sont identiques (`git diff --stat v0.19.0 v0.19.1` vide) ;
  `grouped_settlements.rs` (dépôt) change, la sonde T0 joue le règlement groupé sur l'image 0.19.1. Le contrôle de
  la perte assumée rend le code 0 depuis les deux tags (`e491d/dev/perte-assumee-773df80da.log`,
  `perte-assumee-v0.19.1.log`).
- **Écartées** : garder l'image 0.19.0 (la recette doit partir de la dernière image publiée, `CLAUDE.md`
  § *Ordre des releases* et `docs/testing.md`) ; réécrire toute la fiche (elle perdrait l'histoire de sa
  validation).
- **Réversibilité** : totale (paramètres).

## C-e491d-2 — Deux comptes bancaires : celui du relevé A, et un principal à IBAN ordinaire

- **Contexte** : sonde T0, premier tirage (`e491d/t0/sonde-t0.log`) : avec l'IBAN du relevé A
  (`CH4431999123000889012`) en compte principal, tout PDF de facture est refusé
  (`400 INVOICE_NOT_PDF_READY`, « QR-IBAN fourni sans référence QRR ») : c'est un **QR-IBAN** (IID 31999). D1 le
  voulait principal.
- **Retenu** : le compte du relevé A reste tel qu'avant la story (non principal, sans compte du grand livre) ; un
  second compte, **principal**, à l'IBAN d'exemple ordinaire de SIX `CH9300762011623852957`, lié au `1020`, porte
  la QR-facture des PDF et le relevé saisi du règlement groupé. La précondition d'AC1 (exactement un compte
  principal, lié au `1020`) est inchangée. Second tirage de la sonde : PDF `200`, règlement groupé accepté.
- **Écartées** : une référence QRR sur chaque facture (le peuplement s'écarterait de l'usage réel le plus courant) ;
  changer l'IBAN du relevé A (il faudrait refaire sa fixture).
- **Réversibilité** : totale.

## C-e491d-3 — Le crédit du règlement groupé est la somme des TTC lus à l'API

- **Contexte** : sonde T0 : `invoices.total_amount` est le **HT** (200.0000 pour 216.20 TTC) ; un crédit égal à leur
  somme fait refuser le groupe (`RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`, écart −32.40).
- **Retenu** : `totalTtc` de F5 et F6 lu à l'API, sommé par `awk` à deux décimales.
- **Écartées** : `SUM(total_amount * (1 + taux))` en SQL (recalculerait l'arrondi de Kesh).
- **Réversibilité** : totale.

## C-e491d-4 — Le script complémentaire compte ses écarts au lieu de s'arrêter au premier

- **Contexte** : D11 exige un rouge prouvant à la fois la comparaison des tables et celle des PDF.
- **Retenu** : `recette-complement-rel0200.sh` arrête seulement sur une impossibilité (port, image, import refusé) ;
  chaque contrôle (nom, version, tables, PDF, F8, journal) est compté, et le code est 1 s'il y a un écart. Son rouge
  se joue sur une **copie** du `$D` de la recette, une ligne d'empreinte et un octet du PDF de F1 altérés : les
  deux écarts paraissent dans un seul tirage. La liste des tables est celle de `$D/journal/colonnes-avant/` (la
  `TABLES` du script, relevée par la phase 1), pour ne pas la recopier.
- **Écartées** : deux tirages, un par altération (une recette de plus sur la voie base).
- **Réversibilité** : totale.

## C-e491d-5 — Les mutations du test shell portent sur une copie de `scripts/`

- **Contexte** : T6 exige une preuve du rouge par scénario, mutation du script du mini-dépôt.
- **Retenu** : `kesh-gate-logs/e491d/mut/mutations.py` copie `scripts/` et `docker-compose.dev.yml` sous
  `mut/racine-<nom>/`, y applique un remplacement exact (présent une fois, sinon montage refusé), lance le test de la
  copie, garde le journal `mut/rouge-<nom>.log` et compte la mutation tuée si l'assertion attendue y échoue ; la
  copie est retirée. Le worktree n'est jamais muté, d'où ni `git restore` ni `touch` à faire.
- **Écartées** : muter le worktree puis restaurer (risque d'un commit muté, d'une restauration oubliée).
- **Réversibilité** : sans objet (hors dépôt).
