# Prompt — validation P4 CIBLÉE, Story 25-4-a (le résiduel juste)

*Versionné le 2026-09-27. **Une lentille** (Sonnet), contexte frais. Passe **ciblée** (CLAUDE.md
§ « La passe ciblée ») : la P3 n'a trouvé que des défauts **de propagation des corrections P1** ; ce qu'il reste à
relire est la remédiation, pas la story entière.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-propager-le-residuel`. Fiche :
`_bmad-output/implementation-artifacts/25-4-a-residuel-juste.md`. **Périmètre** : le diff aplati
des remédiations P3 et de l'arbitrage du 2026-09-27 —
`git diff 1344b3a2 b18f7cd2 -- _bmad-output/implementation-artifacts/25-4-a-residuel-juste.md _bmad-output/implementation-artifacts/25-4-propager-le-residuel.md`
(ignore les paragraphes de la fiche mère qui consignent les arbitrages de Guy sur 25-4-b : **ne pas
les contester**).

## La lentille : chasseur de régressions de remédiation

Pour chaque passage réécrit en P1 ou P2 :

1. **Est-il vrai contre le code ?** Relis les sites qu'il cite : `get_invoice`
   (`crates/kesh-api/src/routes/invoices.rs`), `docs/api-external.md`, le bouton et le dialogue
   d'avoir (`frontend/src/routes/(app)/invoices/[id]/+page.svelte`), la fixture `monter` et
   `la_precedence_de_l_annulation_lecture_et_ecriture` (`crates/kesh-db/tests/invoice_settlement.rs`),
   la migration `20260827000001_invoice_settlements.sql`, `settlement_cancel_blocker`
   (`invoice_settlements_write.rs`), `kesh-api/Cargo.toml` (`serde-str`).
2. **Le gabarit « détacher, créditer, rattacher » (AC 13)** : exécute-le **mentalement** pas à pas
   contre `create_credit_note` et les quatre paires combinées de `monter`. Une FK, une contrainte,
   un audit, un contrôle de `company_id`, une lecture de `invoice_settlements` par
   `create_credit_note` ou par les motifs montés **après** peut-il le casser ? L'écriture de
   règlement porte-t-elle quelque part l'identifiant de la facture d'origine (description, audit,
   `invoice_settlements` seul) ?
3. **AC 14 révisé** : cohérent avec l'AC 11, l'AC 12, les tests de l'AC 16 et la mutation m6/m7 ?
   Un test de l'AC 16 porte-t-il encore l'ancienne logique (« bouton absent ») ?
4. **AC 8 révisé** : l'assertion prescrite peut-elle passer à vide (`null`, champ absent) ?
5. **Propagation** : chaque correction a-t-elle été reportée partout où le symptôme était écrit
   (tâches T1-T9, tableau « Où regarder », Change Log, fiche mère, `sprint-status.yaml`) ? Greppe la
   **valeur** (`code()`, `:98`, `937-990`, `latent`, `masqué`), pas la formulation.

## Ce que tu rends

- **Findings** : sévérité, endroit exact, **preuve** (commande et résultat, code lu), correction.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`cargo check`.

## Complément P4

Le diff contient aussi l'**arbitrage du 2026-09-27** (refus de l'avoir maintenu, levée tracée par
#471 ; #384 ramenée en 25-4-d) : **ne pas le contester**, mais vérifier que la fiche 25-4-a reste
cohérente avec lui (AC 12, AC 17, hors périmètre de la fiche mère), et que la fiche de l'epic et
`sprint-status.yaml` disent la même chose. Vérifie aussi que les corrections P3 (T7 « m1 à m7 »,
commentaire `:842-844`, doc-comments `errors.rs:214-217` et `invoice_settlements_write.rs:326-329`,
repli « arrêter et demander », T2 et le montage à 8,1 %) sont justes et ne contredisent rien.
