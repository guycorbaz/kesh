# Prompt — passe 4 de `bmad-create-story validate`, Story 25-3-a-1

*Versionné le 2026-09-24. Deux lentilles en contexte frais (Sonnet) — cycle Sonnet → Haiku → Opus,
reprise après Opus. La numérotation continue celle de la mère (`25-3-a`, trois passes).*

Ton objet est `_bmad-output/implementation-artifacts/25-3-a-1-annuler-reglement-client.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-3-a-annuler-reglement`.

Contexte : la fiche **mère** `25-3-a-annuler-reglement.md` (`split`) a subi trois passes ; la passe 3
a remonté 1 HIGH et une dizaine de MEDIUM (cf. son Change Log), et déclenché le découpage en
**25-3-a-1** (client, ton objet) et **25-3-a-2** (fournisseur). La 25-3-a-1 est **neuve** : elle
réécrit la partie client en intégrant les corrections. ⛔ **Motif mesuré : la sévérité se déplace
vers ce qu'on vient d'écrire** — tout ce qui est neuf ici (l'AC 3 et sa table, `SettlementCancelBlocker`,
`DbError::SettlementNotCancellable`, les champs `cancelBlocked*`, la famille de clés, le test de
composition) est le premier suspect.

⚠️ **Ne conteste pas les arbitrages de Guy** (section « Arbitrages de Guy qui s'appliquent ici »),
dont **Q5** (règlement d'exercice clos → refus, rouvrir l'exercice est le chemin). Conteste leur
**mise en œuvre**.

## Lentille A — le dépôt et la précédence

1. **La table de l'AC 3** : chaque rang est-il **produisible** par un vrai chemin de l'application,
   chaque paire aussi ? Un rang en masque-t-il toujours un autre ? La colonne « à l'écriture »
   est-elle juste — qui refuse, avec quelle erreur, et le 400 qui nomme les comptes est-il bien
   préservé ? `INVOICE_CREDITED` : le statut `cancelled` ne naît-il vraiment que de l'avoir en
   production (vérifie par `grep`) ?
2. **« Une seule garde par motif »** : l'AC 3 dit que le geste ne refuse que les rangs 1-2 et laisse
   le socle refuser les rangs 3-5. Est-ce **vrai** dans le socle actuel pour chacun (rang 4 :
   `find_open_covering_date` ; rang 5 : étape 3 de `reverse_in_tx`) ? La mutation de l'AC 11
   (appel direct à la variante) prouve-t-elle réellement la garde du rang 3 ?
3. **Le geste (AC 4-6)** contre `settle_invoice`, `accept_one_invoice`, `credit_notes.rs` : ordre des
   verrous, `paid_at`, `version`, statut ; ce que la 25-3-b en héritera.
4. **Tout lecteur de l'état** qu'une annulation modifie et que la fiche n'aurait pas vu.
5. **Exactitude** : chaque `fichier:ligne`, fonction, décompte — depuis la source.

## Lentille B — l'API, les textes, l'écran, les tests, le manuel

1. **Les routes et les champs (AC 8)** : `cancelBlockedLabel`, `cancelBlockedDocumentId` — le
   frontend peut-il tout ce que l'AC 9 et l'AC 10 exigent avec ce qu'on lui donne ?
2. **La famille `invoices-settlement-cancel-blocked-*`** : passe-t-elle réellement
   `frontend/scripts/lint-i18n-ownership.js` depuis `features/invoices/` (lis le script) ? Et les
   pages de `src/routes/` qui l'utilisent ? Les gardes vitest i18n : que diront-elles ?
3. **L'inventaire des commentaires #414 (AC 10)** : refais le `grep` et compare ; le tri client /
   fournisseur est-il juste ?
4. **Les tests (AC 11)** : un test prescrit passe-t-il à vide, ou porte-t-il sur un état impossible ?
   Le test de composition prouve-t-il ce qu'il dit ?
5. **Le manuel (AC 12)** : liste **close** des passages, contrôlée sur le **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`). Cherche aussi
   `README.md`, `website/`, `docs/api-external.md`.
6. **Frontière avec la 25-3-a-2** : rien ne tombe entre les deux, rien n'y figure deux fois en se
   contredisant (lis aussi `25-3-a-2-annuler-reglement-fournisseur.md`).

## Ce que tu rends

- **Les findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), l'endroit exact, **la preuve**, ce
  qu'il faut changer.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`.
Lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` (vers le scratchpad) et
`cargo check` sont autorisés.
