# Prompt — validation P3 de la fiche 63-1 (#1752, #1753, #1754, P1), passe complète, modèle Sonnet

Même mandat, mêmes interdits et mêmes axes A1 à A10 que `63-1-annulations-datees-origine-validation-prompt-p1.md` (à relire en entier), contexte
frais. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e63/63-1-validation-p3.md`. Code de référence : `6b71d4c21` (le mécanisme de la 53-6a) ; la fiche
est au HEAD de la branche. Langue : français. Les décisions Q1 à Q6 (53-6a) et Q-A à Q-D (Project Lead, 2026-11-27) ne se rouvrent pas.

Passe **complète** (et non ciblée) : le MEDIUM de la P2 n'est pas né du correctif de la P1 ; il vient d'un axe que les passes précédentes n'avaient
exercé qu'en partie (A7, les attendus des tests existants).

## La P2 (Opus, 2026-11-27) : 0 C, 0 H, 1 M, 6 L, tous corrigés dans la fiche — à contrôler, non à croire

- **V3 (M)** : AC-11 gagne « Autres tests qui changent d'attendu » : `jour_suisse_nuit_e2e::ac7_les_services_jugent_au_jour_suisse` (`:941`),
  `lettering_documents::locked_period_supplier_cancels_are_refused_until_unlocked` (`:1675`), et trois harnais de précédence qui lisent sous `Today`
  et cliquent par le geste public (`invoice_settlement.rs:1475/1555`, `supplier_invoices_repository.rs:1762/1806`, `:2076/2102`), avec une
  prescription (branche `Today` par la forme `_in_tx`) et une méthode de clôture à T1/T2 (inventaire de chaque rouge au Dev Agent Record).
  **Cherche ce que cette liste omet encore** : tout test de `crates/*/tests/` qui appelle `invoice_settlements_write::cancel_settlement`,
  `supplier_invoices::{cancel, cancel_in_tx, cancel_settlement}` ou les trois routes, ou lit `GET /invoices/{id}/settlements` / la vue fournisseur,
  sous un montage où la date de l'origine diffère du jour **et** (i) une borne la couvre, ou (ii) aucun exercice ouvert ne couvre le jour, ou (iii)
  l'inverse est comparée au jour ; montages dans des auxiliaires compris (`monter`, `monter_achat`, `societe`, `tests/support/*`, `tests/common/*`).
  La prescription des harnais garde-t-elle la queue `Today` testée (D7) sans rendre un test vrai par construction ?
- **L** : recompte de D3 (230 sites, 35 fichiers, `cancel_in_tx` compris — recompte toi-même, `git grep -o … | wc -l`, et dis ce que le motif
  omet encore) ; AC-10 réécrit en inventaire clos par fonction englobante (`tests/lexique/mod.rs`) avec trois occurrences attendues et la fermeture
  des alias — est-il constructible avec les fonctions existantes du module, l'égalité stricte se tient-elle à `6b71d4c21` une fois la story faite ?
  l'angle mort `#[cfg(test)] mod x;` est-il juste (`kesh-report/src/lib.rs:26`) ? ; T2 (liste des enveloppes) ; AC-8 e
  (`lettering_documents.rs:814`) ; AC-12 (`supplier_invoices_repository.rs:2209`, course avec `lock_books` écrite en angle mort) ; issue proposée.

## Axes exercés par la P2 (à refaire de façon indépendante)

A1 (P3, P4 au code, routes et enveloppes), A2 (inventaire R1–R11, garde), A4 (ordre des verrous en lecture, course `lock_books`), A5/A7 (tests qui
figent l'ancien contrat, en partie), A8 (`.tex`, PDF aplatis, `api-external.md` lignes citées), A10 (modules, cascade). **Non exercés par la P2** :
A3 (datation des lettrages, devise, écart — repris de la P1 sans relecture), A6 (textes, quatre locales, ownership, écran au clic forcé), A9 (Q-A à
Q-D, prémisses), les mutations M1 à M8 une à une, les notes de version. Exerce-les en priorité.

Rends les findings CRITICAL/HIGH/MEDIUM/LOW avec preuve (ligne copiée ; un finding CRITICAL/HIGH qui affirme une absence se prouve par `grep -nF`),
les axes exercés et non exercés, et le décompte. Dis si la fiche **recycle** (règle de splitting préventif). Écris le prompt de la P4 seulement s'il
reste un finding au-dessus de LOW (modèle : Opus, ou Haiku si ciblée). Aucune écriture hors du rapport ; ni `cargo`, `npm`, `npx`, `docker`, `gh` en
écriture, `scripts/prepare-release.sh` ni aucun script du dépôt ; jamais la base du NAS ; jamais `pkill`/`killall`/`kill` par motif.
