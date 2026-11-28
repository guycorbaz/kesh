# Prompt — validation P2 de la fiche 63-1 (#1752, #1753, #1754, P1), passe complète, modèle Opus

Même mandat, mêmes interdits et mêmes axes A1 à A10 que `63-1-annulations-datees-origine-validation-prompt-p1.md` (à relire en entier), contexte
frais. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e63/63-1-validation-p2.md`. Code de référence : la branche à son HEAD et `origin/main` `0e99cd2db`.
Langue : français. Les décisions Q1 à Q6 (53-6a) et Q-A à Q-D (Project Lead, 2026-11-27, section « tranchées » de la fiche) ne se rouvrent pas.

## La P1 (Sonnet, 2026-11-27) : 0 C, 0 H, 2 M, 4 L, tous corrigés dans la fiche — à contrôler, non à croire

- **V1 (M)** : D3/P4/T2 omettaient `supplier_invoices::{validate, unvalidate, unvalidate_with_policy, pay}` (enveloppes de `commit_with_view`) et leurs routes
  et `_once` ; ajoutés, avec 226 sites de tests dans 34 fichiers. Recompte la commande donnée (périmètre : `crates/*/tests/*`), cherche ce que la liste
  **omet encore** (MCP, `kesh-api/src/services`, `exports`, `inbox_import`, tests de structure lisant les signatures comme
  `supplier_invoice_response_e2e.rs`, `services_rollout.rs`, `date_du_jour_garde.rs`).
- **V2 (M)** : la garde lexicale d'AC-10 vise désormais la **construction** `CancellationDating::Today(` hors motifs (`journal_entries.rs:3192`, `:3208`,
  `settlement_cancellation.rs:228`) et exige un retrait des tests **par bloc** (`#[cfg(test)]` dès `journal_entries.rs:3322`). Est-elle constructible, la
  sonde positive suffit-elle, un site de construction se cache-t-il derrière un alias, un `use`, un appel à plusieurs lignes ?
- **L** : `:1831` n'est pas une création (seule : `:2308`) ; `pay` accepte un paiement antérieur à la facture (montage : valider, payer, puis verrouiller) —
  vérifie qu'un verrou posé **après** le paiement est bien accepté (`books-lock`) et que la vue fournisseur rend alors `PERIOD_LOCKED` ; renvois R1–R6 ;
  chemin d'un inventaire.

## Axes exercés par la P1 (à refaire de façon indépendante) : A1, A2, A3 (lecture), A5, A6 (textes, écran en lecture), A8 (.tex, PDF aplatis,
api-external), A9, A10. Non exercés par la P1 : A4 (ordre des verrous : lu seulement `cancel_in_tx` et `cancel_settlement_in_tx`, sans rejouer `close`/`reopen`/
`lock_books`), A7 en partie (mutations M1 à M8 non rejouées une à une ; la tenue de AC-12 sans `sleep`), le rendu des deux fiches Svelte au clic forcé.

Rends les findings CRITICAL/HIGH/MEDIUM/LOW avec preuve (ligne copiée), les axes exercés et non exercés, et le décompte. Écris le prompt de la P3 seulement s'il
reste un finding au-dessus de LOW. Aucune écriture hors du rapport ; ni `cargo`, `npm`, `docker`, `gh` en écriture, `prepare-release.sh` ; jamais `pkill`/`killall`/`kill` par motif.
