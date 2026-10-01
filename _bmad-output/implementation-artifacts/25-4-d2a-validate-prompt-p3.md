# Prompt — validation P3, Story 25-4-d2a (l'écriture de solde et son annulation)

*Versionné le 2026-10-01. **Une lentille** (Sonnet), contexte frais. P1 (Sonnet) : 2 HIGH, 4 MED, 2 LOW ; P2 (Opus) : 3 HIGH, 1 MED, 9 LOW, dont la réfutation d'un reclassement de P1. Les corrections de P2 sont dans le commit `eaa7d22f` et au Change Log de la fiche — relis-les d abord : une remédiation introduit souvent le défaut suivant.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2a-ecriture-de-solde`. Fiche :
`_bmad-output/implementation-artifacts/25-4-d2a-ecriture-de-solde.md`. Issues : `gh issue view 384`, `gh issue view 490`,
`gh issue view 390`. Patrons : `settle_invoice` et `cancel_settlement_in_tx`
(`crates/kesh-db/src/repositories/invoice_settlements_write.rs`), l'avoir (`credit_notes.rs`). Règles : `CLAUDE.md`
(§ *Migration breaking policy*, § *Pattern batch*). Checklist : `.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages et les choix « retenus par défaut » sont **acquis** : conteste la mise en œuvre, pas le principe. ⚠️ La
fiche décrit du travail **à faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **La migration (AC 1)** : la recréation des deux CHECK est-elle faisable en MariaDB 10.11 (`DROP CONSTRAINT` /
   `ADD CONSTRAINT` dans un même `ALTER`) ? Le verdict « non breaking » tient-il : relis tout ce qu'un binaire
   antérieur fait d'une ligne `write_off` (lecture d'entité, `SettlementChoice`, liste, export CSV, sauvegarde et import
   — `check_schema_compat`, `parse_and_verify` —, annulation, rapprochement, `reconciliation_cancel.rs`). Une colonne
   `JSON` en MariaDB : contrainte `JSON_VALID` implicite, et comment l'export CSV, la sauvegarde et le squash la
   traitent-ils ? Le triage P7 « sans objet » est-il juste ?
3. **Le prorata (AC 2)** : la formule est-elle comptablement juste (réduction de contre-prestation, LTVA art. 41 —
   la TVA corrigée suit la part de chaque taux) ? `total_ttc` doit-il inclure l'arrondi figé ? Que se passe-t-il pour
   une facture réglée en partie (le prorata porte sur le **reste**, pas sur le TTC) — la formule `amount × vat_r /
   total_ttc` est-elle encore juste ? Le reliquat de centimes : où va-t-il, et l'écriture reste-t-elle équilibrée au
   centime près ? Une ligne `base_ht` est-elle utile au débit, ou seulement à la d2c ?
4. **Le compte (AC 3)** : généraliser `rounding_account_for_write` sans changer ses deux messages (`Payment`,
   `Issuance`) ni ses appelants ? La colonne choisie par nature, sans SQL dynamique dangereux ?
5. **L'écriture (AC 4)** : l'ordre des verrous est-il le même que `settle_invoice` (interblocage avec un règlement ou
   une acceptation de rapprochement concurrents) ? L'invariant `version + 1` est-il tenu ? Le seuil `A < 0.05` de la
   nature `rounding` est-il cohérent avec les restes que produisent l'arrondi à 5 centimes et le paiement au centime ?
   La nature `rounding` sur une facture **sans** compte d'arrondi ? Un solde sur une facture qui a déjà un avoir
   (statut `cancelled` → refusé) ? Le libellé « Solde facture {numéro} — {nature} » : langue de la nature ?
6. **La route, la liste, l'annulation (AC 5, AC 6)** : les blocages d'annulation (`settlement_cancel_blocker`) valent-ils
   pour un solde (un solde n'est jamais rapproché) ? Faut-il un rejeu sur interblocage (#491) ? Les erreurs nouvelles
   sont-elles toutes mappées ? Le registre d'audit des routes et les libellés d'audit (`audit_labels.rs`) ?
7. **Les tests (AC 8)**, **le CHANGELOG (AC 7)**, **le périmètre** (modules recomptés : la story reste-t-elle sous le
   seuil de cinq ?), et **ce qui manque** : un site qui lit `invoice_settlements` et qui, devant un `write_off`, se
   tromperait (`grep -rn "invoice_settlements" crates/ --include=*.rs | grep -v tests`) — balance âgée, rapprochement,
   rappels (« déjà réglé » sur le PDF de rappel : un escompte y apparaîtrait-il comme un paiement ?), échéancier,
   tableau de bord.

8. **Les corrections de la passe 2** (`git show eaa7d22f`) :
   - le **nouveau motif de refus d'annulation** (AC 6) : bloque-t-il trop (annuler le solde lui-même doit rester possible ;
     une facture avec deux soldes ?) ou trop peu (le dé-rapprochement, l'acceptation de rapprochement qui ajouterait un
     règlement **après** un solde — la facture est payée, est-ce refusé ?) ? Où se place-t-il dans l'ordre des rangs, et le
     frontend, qui mappe les codes de motif (`settlement-cancel-blocked.ts`), affiche-t-il un code inconnu sans planter
     jusqu'à la d2b ?
   - la **`version`** : la lit-on sous le verrou ? Le rejeu reste-t-il utile s'il échoue en 409 ?
   - **`SettlementKind`** dans `create_in_tx` : compatible avec le côté fournisseur et avec `routes/reconciliation.rs:1595` ?
   - le **refus `paid_at` posé** : contredit-il l'arbitrage « toute facture validée non soldée » ? Une facture payée
     par l'ancien chemin et qu'on voudrait solder — refus voulu ?
   - **l'invariant reformulé** (« tant qu'un solde existe, la facture est payée ») : énumère **tous** les écrivains de
     `paid_at` et de `invoice_settlements` (`grep -rn "paid_at\s*=\|DELETE FROM invoice_settlements\|INSERT INTO invoice_settlements" crates/ --include=*.rs`)
     et dis pour chacun s'il peut le casser.

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie, ou code cité), correction. ⛔ La liste des axes
exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git log`/`show`/`diff`,
`gh issue view`, `cargo check`.
