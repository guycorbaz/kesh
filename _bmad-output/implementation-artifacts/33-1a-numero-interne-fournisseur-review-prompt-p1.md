# Revue de code P1 — 33-1a, le numéro interne des factures fournisseur (#926), trois lentilles B, E, A (lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **33-1a** de Kesh (issue #926, P1). Contexte frais.
Travaille en français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche ce que le code porte en lui-même (SQL de la migration, verrous, erreurs avalées, données perdues, concurrence, rejeu).
- **E — Edge Case Hunter** : chaque branche et chaque frontière — statuts, année de début, deux exercices d'une année, exercice clos ou créé après coup, format changé, 1 000ᵉ facture, base vide, base peuplée, sauvegarde 0.17.4 et 0.17.5, interblocage, brouillon numéroté redaté ou supprimé.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC, mutation par mutation, manuel `.tex` **et PDF**, CHANGELOG, audit d'idempotence, registre.

⚠️ **La migration s'exécutera sur les livres réels de Guy, en production** (Kesh 0.17.4 sur le NAS). Une erreur y coûte des saisies, non une base de dev. Note en conséquence.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e331 log --oneline bdc3faa4..<dernier commit de la 33-1a, à relire au Change Log de la fiche>
git -C /home/gcorbaz/devel/kesh-e331 diff bdc3faa4 <ce commit> -- crates docs CHANGELOG.md
```

Lis le diff **aplati**. Fichiers principaux : `crates/kesh-db/migrations/20261106000001_supplier_invoice_internal_number.sql`, `crates/kesh-db/src/post_restore/20261106000001_supplier_invoice_internal_number.sql`, `crates/kesh-db/src/post_restore.rs`, `crates/kesh-db/src/repositories/supplier_invoices.rs` (`attribute_internal_number_in_tx`, `validate_in_tx`, `document_long_label`, `pay_in_tx_as`), `crates/kesh-db/src/repositories/company_invoice_settings.rs`, `crates/kesh-core/src/invoice_format.rs`, `crates/kesh-api/src/routes/{company_invoice_settings,supplier_invoices,fiscal_years}.rs`, `crates/kesh-api/src/mcp/outils/factures_fournisseur.rs`, `crates/kesh-api/src/exports/csv_tables.rs`, et les tests `crates/kesh-db/tests/{supplier_invoices_internal_number,migrations_upgrade_path}.rs`, `crates/kesh-api/tests/{admin_full_import_e2e,rejeu_interblocage_e2e,supplier_invoice_draft_e2e,mcp_outils_e2e}.rs`.

Fiche : `/home/gcorbaz/devel/kesh-e331/_bmad-output/implementation-artifacts/33-1a-numero-interne-fournisseur.md` (lentille A ; B la lit **après** son examen à l'aveugle, pour classer). Registre : `e33-choix-autonomes-e331.md`, décisions de Guy : `_bmad-output/planning-artifacts/epic-33-numerotation-et-defauts.md`. Journaux : rouges `/home/gcorbaz/devel/kesh-gate-logs/e331/rouges.log`, mutations `/home/gcorbaz/devel/kesh-gate-logs/e331/mut/M*.log`, gates `/home/gcorbaz/devel/kesh-gate-logs/e331/gate-*.log`, répétition `/home/gcorbaz/devel/kesh-gate-logs/e331/repetition-*.log`.

## Axes obligatoires

1. **La migration sur une base peuplée** : ordre des instructions, rangs, partition par société et année de début, `LPAD` à la millième, exercice lu sur l'écriture, `updated_at` et `version` préservés, aucune ligne de `journal_entries` touchée, `UNIQUE` en dernier, tenue si l'`UPDATE` échoue à mi-chemin. Comportement de `UPDATE … JOIN` sur MariaDB 10.11.
2. **Cohérence migration / extrait de rejeu** : verbatim, sentinelle, classe B, fenêtre, ce que voit une sauvegarde 0.17.4 et une 0.17.5.
3. **Les verrous de `validate_in_tx`** : l'étape (3 bis), l'instantané, l'ordre, le verrou d'intervalle, les cycles avec clôture, réouverture et création d'exercice, le rejeu de `reopen`.
4. **Non-réémission** : la preuve de `C-e331-16` et `C-e331-23` contre le code ; un quatrième chemin d'utilisateur ?
5. **Les sites non résolus** : tout `format!` de libellé de facture fournisseur, toute liste de colonnes de `SupplierInvoice` et `CompanyInvoiceSettings` (`git grep`), tout `query_as::<_, SupplierInvoice>`.
6. **Le manuel** — `docs/manual/fr/admin-manual.tex` **et** `docs/manual/fr/admin-manual.pdf`, section `sec:maj-0-17-5`, *Compatibilité*, *Importer*, la phrase sur les reprises rejouées : contrôle le PDF aplati (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `/usr/bin/grep -oF`). Le manuel dit-il vrai contre le code et contre le journal de la répétition ?
7. **Le CHANGELOG et `docs/api-external.md`** contre le code ; la révision des bornes MCP (`C-e331-27`).
8. **P1 à P8** : non breaking, audit d'idempotence recompté, `migrations.sha384`, squash régénéré, triage P7, sites P6.

## Contraintes

⛔ Lecture seule : aucune écriture hors de ton rapport, ni redirection ni fichier temporaire, aucune compilation ni exécution de test, `scripts/prepare-release.sh` et `git checkout`, `git stash`, `docker` interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport : `/home/gcorbaz/devel/kesh-gate-logs/e331-review-p1-{B,E,A}.md`, selon ta lentille. Il porte :

- **les axes réellement exercés et ceux qui ne l'ont pas été**, sans quoi un « 0 finding » ne compte pas ;
- chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait lu, scénario, remédiation ;
- toute affirmation de présence ou d'absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ;
- un décompte recompté.
