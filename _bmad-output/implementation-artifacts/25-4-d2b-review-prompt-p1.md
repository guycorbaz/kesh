# Prompt — revue de code P1, Story 25-4-d2b (solder le reste : le bouton, le dialogue, la liste)

*Versionné le 2026-10-02. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2b-bouton-solder-le-reste`. **Diff à revoir : `git diff 0ddfc3f2 ff57fc9b`** —
un seul commit d'implémentation. Fiche : `_bmad-output/implementation-artifacts/25-4-d2b-bouton-solder-le-reste.md`
(AC 1–7, *Limites assumées*, Dev Agent Record). L'API qu'elle appelle : `crates/kesh-api/src/routes/invoices.rs`
(`write_off_invoice_handler`), `crates/kesh-db/src/repositories/invoice_settlements_write.rs` (`write_off_invoice`).
Issues : `gh issue view 490`, `496`, `497`. Règles : `CLAUDE.md`.

## Lentilles

- **A — Blind hunter** : le diff **seul**, sans la fiche. Ce qui est faux ou fragile : réactivité Svelte 5 (`$state`,
  `$derived`, `$effect` — boucles, états périmés, effets qui se déclenchent trop ou pas assez), arithmétique `big.js`
  (aucun `Number` sur un montant ?), conditions d'affichage, gestion d'erreur, typage.
- **B — Edge-case hunter** : le diff et le code environnant.
  - Les états de la fiche : facture brouillon, annulée, payée sans ligne, reste nul, `amountDue` à `null`, réglages
    inconnus, liste en chargement puis en échec, double clic sur « Solder », fermeture du dialogue pendant l'envoi.
  - La condition « fraction de centime » et le seuil de 0.05 : reproduisent-ils **exactement** le serveur
    (`invoice_settlements_write.rs`) pour 0.0049, 0.0050, 0.0450, 0.0500, 10.0050, 68.1000 ?
  - Après un refus : chaque code d'erreur du serveur (`WRITE_OFF_ACCOUNT_NOT_CONFIGURED`, `INVALID_INPUT`,
    `OPTIMISTIC_LOCK_CONFLICT`, `ILLEGAL_STATE_TRANSITION`, `CONFIGURATION_REQUIRED`, `FISCAL_YEAR_INVALID`,
    `PERIOD_LOCKED`, erreur réseau) : le dialogue fait-il ce que dit la fiche ? La relecture qui **échoue** ?
  - Le récapitulatif : `amountSettled − soldé` peut-il être négatif ou faux (liste et facture d'instants différents) ?
  - L'E2E : son nettoyage tient-il si une assertion échoue au milieu ? Laisse-t-il une facture qui gêne une autre spec ?
- **C — Acceptance auditor** : chaque AC tenu ? `sitesTotal`, compteur des libellés en dur, clés i18n dans les 4
  locales (parité, replis identiques au FTL fr-CH) — recomptés depuis la source. **Pars du symptôme** :
  `grep -rn "settlementType\|amountSettled\|formatInvoiceTotal(.*amount" frontend/src` — un écran qui affiche un montant
  de règlement ou de reste et qui devrait passer par `formatExactAmount` ? Les tests prouvent-ils ce qu'ils disent
  (chacun nomme une mutation : est-elle réellement tuée par l'assertion écrite) ? Le manuel (`docs/manual/fr/*.tex` et
  **PDF aplatis**, `pdftotext … | tr '\n' ' ' | tr -s ' '` vers
  `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`) dit-il ce que fait
  l'écran (rôle, natures, seuil, fraction de centime, annulation) ? Le CHANGELOG ? Le Dev Agent Record n'affirme-t-il
  que ce qui a tourné ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu), scénario d'échec,
correction. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF`. ⛔ La liste des
axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` et expériences dans le scratchpad.
