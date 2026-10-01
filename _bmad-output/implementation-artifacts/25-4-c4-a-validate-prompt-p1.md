# Prompt — validation P1, Story 25-4-c4-a (l'arrondi à 5 centimes, figé à la validation)

*Versionné le 2026-10-01. **Une lentille** (Sonnet), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c4-arrondi-facture-5-centimes` (empilée sur la c3-b,
PR #493). Fiche : `_bmad-output/implementation-artifacts/25-4-c4-a-arrondi-fige-a-la-validation.md`. CR :
`gh issue view 494` (arbitrages de Guy). Sœurs : `25-4-c3-b-arrondi-au-centime.md`,
`25-4-c3-a1-reglage-compte-arrondi.md`. Règles : `CLAUDE.md` (§ *Migration breaking policy* P1–P8,
§ *Pattern batch*). Checklist : `.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages de Guy (réglage par société actif par défaut, avoirs idem, ligne sans TVA, écart à la
validation sur le compte d'arrondi, découpage c4-a / c4-b) sont **retenus** : conteste la mise en œuvre,
pas le principe. ⚠️ La fiche décrit du travail **à faire** : ne reproche pas au code de ne pas le porter.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **L'inventaire des formes du TTC — pars du symptôme** :
   `grep -rn "invoice_total_ttc\|INVOICE_TTC\|line_ttc_sql!\|INVOICE_CREDITED\|total_ttc" crates/ frontend/src`.
   Chaque site qui calcule ou lit le TTC d'une facture client ou d'un avoir est-il dans l'AC 5, ou exclu
   avec sa raison ? Et les sites qui lisent les **lignes** pour en refaire une somme (rapport TVA,
   `kesh-report`, exports CSV, `admin.rs` import, `kesh-seed`) ?
3. **La règle (AC 1)** : la stratégie d'arrondi au 0.05 « loin de zéro » est-elle exprimable avec
   `rust_decimal` sans recopier une stratégie déjà définie ailleurs ? Les exemples sont-ils justes ?
4. **La migration (AC 2)** : réellement non breaking (P1) — un binaire antérieur lit-il ces tables par
   `SELECT *` ou par une liste de colonnes ? P5, P6 (`grep -rn "migrations.len()\|apply_migrations_up_to" crates/`),
   P7 (une migration DDL avec `DEFAULT` déclenche-t-elle le détecteur ?), P8 ; squash et
   `test_schema_guard` ; `migrations.sha384`. Une sauvegarde **antérieure** (sans les colonnes) s'importe-t-elle
   — et que valent alors `rounding_amount` et le réglage ?
5. **La validation (AC 3)** : l'ordre des étapes de `validate_invoice` (verrou, réglages, refus à zéro,
   matérialisation des comptes, écriture, `UPDATE`) permet-il de calculer l'arrondi et de lire le compte
   avant toute écriture ? `unvalidate` : l'arrondi remis à 0 suffit-il (écriture supprimée ou
   contre-passée ?) ?
6. **L'écriture (AC 4)** : les trois lecteurs « première ligne au débit » — en existe-t-il d'autres qui lisent
   l'écriture de vente par position ou par montant (backfill `20260729000001…` condition (3) « crédit égal à
   `total_amount` », rejoué à la restauration par `post_restore.rs`) ? La ligne d'arrondi au **crédit**
   casse-t-elle ce backfill ou une autre lecture ?
7. **L'avoir (AC 6)** : `create_credit_note` peut-il lire et recopier l'arrondi ; l'écriture d'avoir et le
   reste dû d'une facture créditée tombent-ils exactement à 0 ? Que devient l'avoir d'une facture
   **antérieure** (arrondi 0) ?
8. **Faisabilité des tests (AC 7)** et **tests existants qui bougent** : combien de tests affirment un TTC non
   multiple de 0.05 (`grep` dans `crates/*/tests`, `frontend/tests/e2e`) ? La consigne « ne pas désactiver
   le réglage pour faire passer » est-elle tenable ?
9. **Périmètre** : modules recomptés depuis les tâches ; la frontière avec la c4-b est-elle nette (rien de
   visible ici qui manquerait de son pendant) ? Le décalage PDF entre les deux stories est-il acceptable ?

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact, **preuve** (commande exécutée et sa
  sortie, ou code lu cité), correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base
(`scripts/*` dont `scripts/prepare-release.sh` et `scripts/regen-test-schema.sh`, `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`/`reset`, `sqlx migrate`, `cargo test`/`nextest`, `npm run`, `npx`,
`gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`cargo check`, expériences hors dépôt dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`.
