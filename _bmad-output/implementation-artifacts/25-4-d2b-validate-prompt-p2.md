# Prompt — validation P2, Story 25-4-d2b (le bouton, le dialogue et la liste du solde)

*Versionné le 2026-10-02. **Une lentille** (Opus), contexte frais. La passe 1 (Sonnet) a rendu 2 HIGH, 3 MED, 2 LOW ; ses corrections sont dans le commit `4bc9cc4b` et au Change Log de la fiche — relis-les d'abord : une remédiation introduit souvent le défaut suivant.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2b-bouton-solder-le-reste`. Fiche :
`_bmad-output/implementation-artifacts/25-4-d2b-bouton-solder-le-reste.md`. Story précédente (l'API) :
`25-4-d2a-ecriture-de-solde.md`. Issues : `gh issue view 490`, `gh issue view 384`. Règles : `CLAUDE.md`. Checklist :
`.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages sont **acquis** : conteste la mise en œuvre, pas le principe. ⚠️ La fiche décrit du travail **à faire** :
un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **Le contrat avec l'API** (d2a, `crates/kesh-api/src/routes/invoices.rs` : `WriteOffInvoiceRequest`,
   `WriteOffInvoiceResponse`, `InvoiceSettlementResponse`) : les noms de champs, la forme de `version` (nombre), les
   valeurs de `nature`, les codes d'erreur — la fiche les reprend-elle exactement ?
3. **Le dialogue (AC 2)** : la règle « reste d'arrondi proposé si reste < 0.05 » est-elle la même que celle du serveur
   (`>= 0.05` refusé) ? Le reste dû affiché au centime alors que le serveur solde le reste exact à quatre décimales —
   l'utilisateur voit-il un montant différent de celui qui sera écrit, et faut-il le dire ? Une nature « sans compte »
   détectée par `invoiceSettings` : le serveur relit aussi le compte **actif et imputable** — l'écran ne verra pas un
   compte archivé ; est-ce assumé ? Le compte d'arrondi requis pour la fraction de centime d'une autre nature (d2a) :
   l'écran le sait-il ? Le 409 : relire la facture suffit-il, ou faut-il fermer le dialogue ?
4. **La fiche (AC 1, AC 4)** : `canManage` sur le nouveau bouton alors que l'ancien ne l'a pas — cohérent ? Le calcul
   « Déjà réglé = amountSettled − soldé » à partir de la **liste** : que se passe-t-il si la liste n'est pas chargée
   (échec toléré, `settlements = []`) — le récapitulatif ment-il ? Les autres écrans qui lisent `amountSettled`
   (échéancier, `paymentStatusOf`) : la limite assumée est-elle juste et complète ?
5. **La liste (AC 3)** : `typeLabel` est recensé par `i18n-libelle-en-dur.test.ts` ; le libellé « Annuler le solde » —
   le dialogue de confirmation d'annulation de la fiche parle-t-il de « règlement » à tort pour un solde ?
6. **Le manuel (AC 6)**, **le CHANGELOG**, **les tests (AC 7 — l'E2E remet-il vraiment son état ?)**, **le périmètre**
   (modules recomptés), et **ce qui manque** : un autre écran qui liste les règlements ou affiche « payée » et qui
   devrait connaître le solde (`grep -rn "settlementType\|amountSettled\|paymentStatusOf" frontend/src`).

7. **Les corrections de la passe 1** (`git show 4bc9cc4b`) : le pré-contrôle du compte d'arrondi « pour toute nature sur
   un reste à fraction de centime » reproduit-il **exactement** la condition du serveur (`amount != amount_due_to_centime(amount)`,
   arrondi `MidpointAwayFromZero`) — avec `big.js`, quelle méthode d'arrondi, et sur quel champ (`amountDue` porte-t-il
   bien les quatre décimales à l'API) ? Le drapeau « liste chargée » : que montre la fiche **pendant** le chargement, et
   après une annulation qui relit la liste ? La lecture de la `version` « au moment de la confirmation » : comment le
   dialogue reçoit-il la facture relue (prop réactive, `$derived`) ? Les nouvelles chaînes du dialogue d'annulation : clés
   neuves ou variantes, et `sitesTotal` ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie, ou code cité), correction. ⛔ La liste des axes
exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git log`/`show`/`diff`,
`gh issue view`.
