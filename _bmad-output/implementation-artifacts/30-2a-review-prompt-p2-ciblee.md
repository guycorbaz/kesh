# Prompt — passe ciblée P2 de la revue de code, Story 30-2a (documents sans QR)

*Versionné le 2026-10-15. **Passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, contexte frais,
**Haiku**, braquée sur le **seul diff de la remédiation de la revue P1** — le report sur `main` (commits antérieurs à
`75020c01` compris) n'en fait **pas** partie. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2`. Le diff à relire, **aplati**, et lui seul :

```sh
git diff 75020c01 HEAD -- . ':!_bmad-output/implementation-artifacts/30-2a-review-prompt-p2-ciblee.md'
```

Rapports qui l'ont motivé : `/home/gcorbaz/devel/kesh-gate-logs/302a-review-p1-B.md`, `302a-review-p1-E.md`,
`302a-review-p1-A.md`. La remédiation est décrite à la fiche
`_bmad-output/implementation-artifacts/30-2-documents-factures-fournisseur.md`, Dev Agent Record § *Report et
remédiation de la revue de code P1*, et à l'entrée datée « Revue de code P1 » du Change Log (une ligne par finding).
Règles : `CLAUDE.md` (§ *La passe ciblée*, § *Une passe qui ne déclare pas ses axes exercés ne compte pas*, § *Haiku-
specific guardrails*, § *Le prompt d'une passe doit NOMMER le manuel*) ; `docs/agents-orchestration.md` § 6.

⚠️ **Lis le diff aplati ci-dessus, pas une suite de commits.** Ce que le dépôt disait avant `75020c01` n'est pas
l'objet de la passe.

## Lentille T — régression de la remédiation P1

Pour **chaque ligne ajoutée** du diff qui affirme un fait (texte d'écran, comportement, chemin, constante, nombre),
vérifie-le au code par `grep -nF` ou `sed -n` et **copie la sortie**. Cherche aussi ce que le patch a pu casser autour
de lui. Axes, tous à exercer :

1. **E1 — le texte « QR illisible »** : la clé `imported-supplier-invoices-qr-unreadable-hint` dit-elle la même chose
   dans les **quatre** locales (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`) et dans le repli de
   `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte` (repli = fr-CH au caractère près) ? Ce qu'elle
   affirme (« la facture créée sera sans IBAN ni référence ») est-il vrai au code de la complétion
   (`crates/kesh-api/src/routes/imported_supplier_invoices.rs`, `complete_import_once`) ? Reste-t-il, **hors entrées
   datées et hors fiche**, une phrase qui promet une saisie à la main de l'IBAN ou de la référence — greper la valeur :
   `grep -rnE "saisissez-les|à saisir à la main|se saisissent à la main|von Hand|a mano|by hand" --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git .` ?
2. **E2 — les tests de course de l'aperçu** (`frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.test.ts`,
   deux tests « course : ») : prouvent-ils vraiment la garde `if (cancelled) return;` de `DocumentPreview.svelte`
   (la lecture du document précédent finit-elle **après** le changement de document ? l'assertion porte-t-elle sur
   ce qu'une mutation changerait ?) ou pourraient-ils passer pour une autre raison (promesse jamais résolue, assertion
   faite avant la fin de la lecture) ? Même question pour le test de stabilité de `previewLoader`
   (`import-page.test.ts`, « ne relit pas le document quand la liste se recharge ») : la liste est-elle **réellement**
   rechargée avec un objet neuf pour la même ligne ?
3. **Manuels et PDF** : `docs/manual/fr/user-manual.tex` (§ *Documents à traiter* : « Quatre gestes », *Télécharger*,
   la phrase sur l'image non décodable — CMJN, 10 000 pixels), `admin-manual.tex` (§ `sec:inbox-import`, § de
   l'export d'installation), `marketing-brochure.tex` : chaque affirmation est-elle vraie au code
   (`MAX_IMAGE_DIMENSION` dans `crates/kesh-api/src/qr_decode.rs`, `data-testid="imported-download"` et son nom de
   fichier, le refus `ImageDecode` → `UNSUPPORTED_FILE_TYPE` de `crates/kesh-api/src/inbox_import.rs`) ? Le **PDF**
   dit-il la même chose que le `.tex` — contrôle **aplati vers la sortie standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "Quatre gestes"` (idem pour les
   autres phrases et les deux autres PDF) ?
4. **Commentaires de code modifiés** (`inbox_import.rs` l. ~13-17 et ~550-556,
   `crates/kesh-db/src/entities/imported_supplier_invoice.rs` l. ~15-20) : disent-ils vrai ? Le diff a-t-il touché
   autre chose que des commentaires dans ces deux fichiers Rust ?
5. **Registre et audit** : l'annotation de C-30-2-4 (`epic-30-choix-autonomes-302.md`) et la précision A2 de
   `docs/migrations-idempotence-audit.md` (les **huit** colonnes déjà facultatives — à confronter à
   `crates/kesh-db/migrations/20260629000001_imported_supplier_invoices.sql`) sont-elles justes ? Le fichier de
   migration `20261015000001_imported_documents_without_qr.sql` est-il **resté intact** dans le diff (P8) ? Les
   compteurs de l'audit (`ls crates/kesh-db/migrations/*.sql | wc -l`, `grep -c '^| \`20' docs/migrations-idempotence-audit.md`)
   tiennent-ils ?
6. **Glossaire** (`docs/i18n-glossaire.md`) : les valeurs qu'il cite pour `-discard`, `-view-doc`, `-doc-gone` sont-elles
   celles des quatre catalogues ?
7. **Fiche et comptes rendus** : la remédiation a-t-elle **réécrit** une entrée passée (Change Log, registre, Record)
   au lieu de l'**annoter** (interdit) ? Les décomptes écrits (tests ajoutés, mutations, gates) se retrouvent-ils dans
   le dépôt et dans les journaux cités (`/home/gcorbaz/devel/kesh-gate-logs/302a-p1rem-*.log`,
   `302a-mutations-p1.log`) ? Recompte : `grep -cE "^\s*it\(" frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.test.ts`.
8. **Mots-clés de fermeture** : `git log 75020c01..HEAD --format=%B | grep -niE "\b(close[sd]?|fix(e[sd])?|resolve[sd]?):?\s+#[0-9]+"` doit être vide.

**Haiku — garde-fous** (`CLAUDE.md`) : tout finding qui affirme l'**absence** d'un patch ou la **présence** d'un
anti-pattern s'appuie sur une sortie `grep -nF` copiée, sur le **fichier final** et non sur une ligne d'un hunk ; ce que
la fiche **prescrit** aux sous-stories 30-2b et 30-2c (saisie de l'IBAN, page de création, document joint) n'existe pas
encore — son absence n'est **pas** un défaut ; ne signale que ce que le diff **affirme** et qui est faux, ou ce qu'il
casse.

## Ce que tu rends

Findings numérotés `T1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, fichier et ligne, **preuve** (commande
exécutée et sortie copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (1 à 8 ci-dessus) — un
« 0 finding » sans elle ne compte pas comme passe. Rapport **écrit d'un seul jet**, un seul fichier, à la fin, sans
brouillon ni fichier intermédiaire : `/home/gcorbaz/devel/kesh-gate-logs/302a-review-p2-T.md`. Dernier message : le
chemin du rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport. Lecture seule stricte : **aucun `git checkout`, `git switch`,
`git checkout --detach` ni aucune autre forme** ; aucune commande qui écrit, compile ou exécute : `scripts/*` (dont
**`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`),
`make`, `latexmk`, `xelatex`, `git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`,
`npx`, `node`, **`python3`, `python`, `perl`, `ruby` et tout autre interpréteur**, `sed -i`, `docker`, **`mariadb`,
`mysql` et toute requête SQL**, `bash` sur un script du dépôt, `gh` en écriture. `pdftotext` vers la **sortie
standard** seulement, jamais vers un fichier. Aucune donnée réelle de Guy ne se lit ni ne se cite (rien de
`~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log`/`show`/`diff`/`blame`,
`pdftotext … -`.
