# Prompt — validation P3 de la fiche v0172-correctifs (release 0.17.2 : #840, #870, #851, #882, #880, #883, #885), lentille complète, Sonnet

Tu valides la **fiche** de la Story v0172-correctifs de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11),
release corrective **0.17.2**, **sans migration**. Depuis la remédiation de sa validation P2 (Opus, 0/0/11/12), **#876 (la
session) n'en fait plus partie** : elle a été extraite dans la fiche sœur `v0172-session.md` (validée à part) ; cette fiche n'en
garde que des renvois. Elle compte **trois sous-stories** :

- **A2** — **#840 (P2)** : deux imports simultanés du même fichier ; remède par le verrou nommé du compte (motif de la saisie
  manuelle), sans contrainte ni migration ; la mutation « verrou rendu avant le commit » a été **retirée**, angle mort déclaré.
- **B** — **#870** (pagination, `?status=` et `?cancelled=1` de la liste des factures fournisseur, lecture **réactive** de l'URL,
  lien du contrôle K8), **#851** (hypothèse de l'issue — rôle nul — réfutée ; cause lue au code : la barre latérale déborde ;
  **oracle du symptôme** dont le résultat décide du correctif, sans et avec bandeau), **#882** (taux de TVA désactivé ; repli `8.10`
  limité aux lignes neuves ; serveur inchangé), **#885** (date du jour locale marquée « à vérifier », échéance qui suit la date).
- **C** — **#880** (dates ISO de onze écrans comptables, des clés API et des taux de TVA ; horodatages **UTC affichés à l'heure de
  Zurich**) et **#883** (six LOW).

Worktree `/home/gcorbaz/devel/kesh-0172b`, branche `story/v0172-correctifs`, partie de `d7ec2ea2` (seuls des fichiers de
`_bmad-output` diffèrent depuis). Langue du rapport : **français**. Modèle : **Sonnet**. **Troisième passe**, **complète**, **une
seule lentille** R (la règle et sa preuve) + F (les textes et ce qu'ils affirment). **Contexte frais** : refais les relevés ; ne lis
les rapports `/home/gcorbaz/devel/kesh-gate-logs/v0172b-validate-p1.md` et `…-p2.md` qu'**après** les tiens, pour l'axe de la
remédiation.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v0172-correctifs.md`, **en entier** (et, pour les renvois seulement, l'en-tête de
   `v0172-session.md`).
2. Le registre `_bmad-output/implementation-artifacts/v0172-choix-autonomes.md`, section `v0172b` (C-v0172b-1 à 28 ; ceux de #876
   marqués « déplacé »).
3. Les issues, en lecture seule : `gh api repos/guycorbaz/kesh/issues/<n>` et `…/<n>/comments` pour 840, 870, 851, 882, 880, 883, 885.
4. Le code — **A2** : `crates/kesh-api/src/routes/bank_imports.rs` (`create`, `create_csv`, `resolve_csv_profile`),
   `routes/bank_statements_manual.rs`, `crates/kesh-reconciliation/src/mutex.rs`, `crates/kesh-api/src/main.rs` (pool),
   `crates/kesh-api/tests/{bank_imports_e2e,bank_statements_manual_e2e}.rs`, `tests/common/mod.rs`,
   `crates/kesh-db/src/test_fixtures.rs`, `lib/features/bank-import/BankImportUpload.svelte`. **B** :
   `routes/(app)/supplier-invoices/+page.svelte` et `supplier-invoices-list-page.test.ts`, `routes/supplier_invoices.rs` (liste),
   `services/factures_fournisseur.rs`, `repositories/supplier_invoices.rs`, `routes/(app)/invoices/+page.svelte`,
   `lib/features/fiscal-years/FiscalYearChecks.svelte`, `routes/(app)/+layout.svelte`, `src/app.css`, les trois bandeaux
   (`DemoBanner`, `IncompleteBanner`, `BackupScheduleBanner`) et l'API de ce dernier, `tests/e2e/sidebar-navigation.spec.ts`,
   `lib/components/invoices/InvoiceForm.svelte` et son test, `routes/(app)/settings/vat-rates/+page.svelte`, `routes/vat.rs`,
   `routes/invoices.rs`, `lib/features/supplier-invoices/{SupplierInvoiceForm.svelte,supplier-invoice-form.ts,SupplierInvoiceForm.test.ts}`,
   `lib/features/invoices/invoice-helpers.ts`, `lib/features/open-items/open-items.ts`, `lib/features/contacts/contacts.types.ts`,
   `routes/(app)/bank-accounts/bank-accounts-page.test.ts:695-720`, `tests/e2e/supplier-invoice-draft-cycle.spec.ts`,
   `tests/e2e/helpers/inbox.ts`. **C** : `lib/features/reports/reports.api.ts`, les 32 fichiers du § 23 et leurs tests,
   `crates/kesh-db/src/pool.rs`, `routes/api_keys.rs`, `crates/kesh-db/src/entities/vat_rate.rs`, `crates/kesh-api/src/qr_decode.rs`,
   `crates/kesh-qrbill/src/{parser,generator}.rs`, `crates/kesh-api/tests/inbox_import_e2e.rs`, `frontend/package.json`.
5. Le **manuel** : `docs/manual/fr/user-manual.tex` (`:815-836`, `:1014`, `:1034-1046`, `:1607-1612`, `:1888-1892`, `:2049-2053`).
   **Le PDF aussi**, lu aplati vers la sortie standard seulement : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
   `docs/manual/{de,en,it}/` : confirme qu'il n'y a aucun `.tex`.
6. Pour contexte : `CLAUDE.md` du worktree ; `docs/agents-orchestration.md` ; `kesh-gate-logs/prompts/v017-donnees-a-garder.md`.

## Axes à exercer

**La remédiation de la P2 — axe obligatoire, à traiter en premier**

- Les findings **P6 à P11 et P18 à P23** de la P2 : le traitement écrit au Change Log (« Validation P2 (Opus) ») est-il **réellement**
  dans la fiche et le registre, et **juste** ? Un remède a-t-il introduit un défaut neuf ? *La sévérité se déplace vers ce qu'on
  vient d'écrire* : relis comme des textes neufs l'oracle du symptôme de #851 (AC-B5, B5 bis, branche « ne rien corriger »,
  `self-start max-h-[…]`), `?cancelled=1` et la lecture réactive de l'URL (AC-B2 bis), la conversion à l'heure de Zurich (AC-C1,
  AC-C2), le § 24 bis, la fixture de #883.5, le retrait de MA13.
- **#851, l'oracle** : reproduit-il le **symptôme** de l'issue (des entrées qui *disparaissent* à l'ouverture d'un groupe, ou en
  ouvrant *Serveur MCP*) ou seulement l'hypothèse ? Peut-il **ne pas** rougir avant correctif (la branche « ne rien corriger »
  est-elle vivante) ? Reste-t-il juste avec un bandeau affiché, après correctif ? `page.mouse.wheel` sur une barre `sticky` sans
  `overflow` fait-il défiler la page, et l'oracle le prend-il en compte ? La forme du correctif (`self-start max-h-[calc(100vh-3.5rem)]`,
  `top-14`, `overflow-y-auto`) tient-elle dans le flex du layout (`:372-380`) ?
- **#880, l'heure de Zurich** : chaque horodatage converti est-il bien de l'UTC (types Rust, `NOW(3)`, `naive_utc()`, session
  `+00:00`) ? Une **date pure** risque-t-elle d'être convertie (et décalée) ? Un champ qui n'est pas UTC (saisi par l'utilisateur,
  p. ex. l'expiration d'une clé API à minuit) s'affiche-t-il encore juste ? Le changement de `formatSwissDate` pour un horodatage
  altère-t-il un site de la 0.17.1 qui l'appelle déjà (cherche tous les appelants : `git grep -n formatSwissDate -- frontend/src`) ?
- **#870, la case dans l'URL** : `?cancelled=1` et `?status=` ensemble, retour arrière, lien du menu, `goto` simulé qui modifie
  `page.url` : décidable et complet ? Les assertions existantes nommées sont-elles toutes, et seulement, celles qui changent ?
- **Les renvois vers `v0172-session`** : la fiche garde-t-elle, hors renvois et entrées datées du Change Log, une trace de #876
  (AC, mutation, tâche, manuel, CHANGELOG) ? Le découpage déclare-t-il juste l'absence de fichier commun entre les deux fiches ?

**La règle et sa preuve**

- **#840** : les deux chemins (`create`, `create_csv`) sont-ils les seuls écrivains d'un relevé depuis un fichier ? L'ordre des refus
  est-il préservé ? Les tests AC-A7 sont-ils réalisables sur le modèle cité ? MA11 et MA12 tuées ? L'angle mort de MA13 est-il bien
  écrit (C-v0172b-28) ? Les limites des Dev Notes (deux, trois connexions ; 409 du rapprochement) sont-elles exactes ?
- **#870** : serveur (`total` filtré, écrêtage), K8 (`FiscalYearChecks.svelte:280-311`), lien et libellé, page au-delà du total.
- **#851** : réfutation du rôle nul ; nombres 18 / 33 / 8 recomptés.
- **#882** : refus serveur à la mise à jour ; les trois sites qui posent `vatRate` ; `_vatPlaceholder` retiré de tout envoi ; message
  sans action inexistante ; repli `8.10`.
- **#885** : prémisse de l'issue (fausse) ; `todayLocal()` et `process.env.TZ` ; règles de l'échéance ; tests existants qui figent la
  date vide (Vitest et E2E).
- **#880** : rejoue l'inventaire (32 fichiers, motifs `M₁`/`M₂`, 64 lignes avant filtre et 51 après sur `d7ec2ea2`, les 20 lignes de la
  liste close), cherche plus large, et vérifie les tests qui figent l'ISO.
- **#883** : chaque point, dont .2 « sans objet » et la fixture de .5 (IBAN remplacé dans le texte produit, `InvalidIban` attendu,
  rouge sous la mutation du parseur).
- **Découpage et périmètre** : A2 trois modules, B cinq, C mécanique ; issues demandées (C-v0172b-19, 20, 25) justifiées ; un point
  écarté sans issue ?
- **Règles du dépôt** : aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 88) ; aucune donnée réelle ; aucun mot-clé
  de fermeture prescrit ; gate et E2E complets au dernier commit de code de chaque sous-story ; captures en thème clair ; i18n
  quatre locales.

**Les textes et ce qu'ils affirment**

- **Le manuel** — axe obligatoire : chaque phrase prescrite est-elle vraie au code après correctif (`:823` pour K8 seulement, `:833`
  inchangée ; pagination ; « de tous les exercices » ; la date « à vérifier » ; la ligne au taux désactivé) ? Une phrase devient-elle
  fausse sans être prescrite ? Le PDF dit-il la même chose ?
- **CHANGELOG** : chaque puce vraie, bien rangée, sans fermeture ; la puce de #851 suspendue à l'oracle ; celle de #880 (heure suisse)
  exacte.
- **Les décomptes** (32 fichiers, 24 sites et le § 24 bis, 31 / 20 lignes, 18 / 33 entrées, 16 lignes UTC, 28 choix, numéros de
  ligne) : recompte-les depuis la source.

## Ce qu'il faut rendre

Un rapport en Markdown à `/home/gcorbaz/devel/kesh-gate-logs/v0172b-validate-p3.md`, **écrit par l'outil d'écriture de fichier
(`Write`), une seule fois**. Il contient :

1. **Les findings**, `P1`, `P2`…, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), sous-story (A2, B, C ou transverse), origine
   (**remédiation P2** ou **défaut d'origine**), lieu (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié)
   et le remède. Une affirmation de présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans cette liste ne
   compte pas.
3. Un décompte final par sévérité, par sous-story et par origine, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture hors du rapport**, et le rapport par l'outil `Write` seulement : **aucune redirection** (`>`, `>>`, `2>`, `&>`,
  `| tee`), **aucune option qui écrit un fichier** (`-o`, `--output`, `pdftotext` vers un fichier), **aucun fichier temporaire** —
  ni dans `/tmp`, ni dans le scratchpad, ni ailleurs. `pdftotext` s'emploie avec `-` (sortie standard) seulement. Un fichier créé par
  erreur se **déclare** au rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit ; `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni `node`, ni
  `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh/`, `~/kesh-test/`, ni le NAS ; aucune donnée réelle dans le rapport.
- Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture, sans
  redirection), `git log`, `git show`, `git diff` (en lecture), `pdftotext … -`, `gh api` en `GET`.
