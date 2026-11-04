# Prompt — validation P2 de la fiche v0172-correctifs (release 0.17.2 : #876, #840, #870, #851, #882, #880, #883, #885), lentille complète, Opus

Tu valides la **fiche** de la Story v0172-correctifs de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11),
release corrective **0.17.2**, **sans migration**, découpée en **quatre sous-stories** dans la même fiche :

- **A1** — **#876 (P2)** : la fiche soutient qu'avec les réglages par défaut (jeton d'accès et inactivité à 15 minutes) le
  renouvellement sur 401 **échoue toujours**, si bien que la session meurt 15 minutes après la connexion quelle que soit
  l'activité ; elle prescrit, **depuis la remédiation de la P1**, un renouvellement **à l'entrée de l'utilisateur**, au plus une
  fois par pas `T`, **sans minuterie**, un champ `sessionExpiresIn` au serveur, un verrou inter-onglets, une garde « 401 sans
  entrée » et la tolérance des échecs transitoires.
- **A2** — **#840 (P2)** : deux imports simultanés du même fichier ; remède par le verrou nommé du compte (motif de la saisie
  manuelle), sans contrainte ni migration.
- **B** — **#870** (pagination et `?status=` de la liste des factures fournisseur, lien du contrôle K8), **#851** (la fiche
  **réfute** l'hypothèse de l'issue — rôle momentanément nul — et attribue le défaut au débordement d'une barre latérale à
  hauteur fixe), **#882** (taux de TVA désactivé sur un brouillon de facture cliente ; serveur inchangé), **#885** (date du jour
  proposée et marquée « à vérifier » sur une facture fournisseur créée depuis un document ; la fiche déclare **fausse** la
  prémisse « l'échéance vient des conditions du fournisseur »).
- **C** — **#880** (dates ISO de onze écrans comptables et de la page des clés API : inventaire clos de 24 sites) et **#883**
  (six LOW ; le .2 déclaré **sans objet**).

Worktree `/home/gcorbaz/devel/kesh-0172b`, branche `story/v0172-correctifs`, partie de `d7ec2ea2`. Langue du rapport :
**français**. Modèle : **Opus**. **Deuxième passe** de validation, **complète** (la P1, Sonnet, a rendu 0/0/12/10, tous appliqués ; rapport
`/home/gcorbaz/devel/kesh-gate-logs/v0172b-validate-p1.md`, à lire **après** tes propres relevés, pour l'axe de la remédiation), **une seule lentille** qui tient à la fois R
(la règle et sa preuve) et F (les textes et ce qu'ils affirment). **Contexte frais** : ne t'appuie sur aucune passe
antérieure, ni sur ce que la fiche dit d'elle-même ; refais les relevés. Les numéros de ligne de la fiche sont ceux de
`d7ec2ea2` (= `HEAD` du worktree, hors fichiers `_bmad-output`).

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v0172-correctifs.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v0172-choix-autonomes.md` (section `v0172b`, C-v0172b-1 à 27).
3. Les issues, **en lecture seule** : `gh api repos/guycorbaz/kesh/issues/<n>` et `…/<n>/comments` pour 876, 840, 870, 851,
   882, 880, 883, 885 (`gh issue view` échoue sur « Projects (classic) »).
4. Le code — **A** : `crates/kesh-api/src/routes/auth.rs` (en entier), `routes/setup.rs` (fin de l'assistant),
   `auth/jwt.rs`, `config.rs` (durées et bornes), `middleware/auth.rs`, `crates/kesh-db/src/repositories/refresh_tokens.rs`,
   `frontend/src/lib/shared/utils/api-client.ts` et `api-client.test.ts`, `frontend/src/lib/app/stores/auth.svelte.ts`,
   `frontend/src/hooks.client.ts`, `frontend/src/routes/login/+page.svelte`, `frontend/src/lib/features/setup/setup.api.ts`,
   `BackupScheduleBanner.svelte`, `api-health.svelte.ts`, `frontend/vitest-setup.ts`, `crates/kesh-api/src/main.rs` (pool) ; `crates/kesh-api/src/routes/bank_imports.rs` (`create`, `create_csv`),
   `routes/bank_statements_manual.rs` (`create`, `create_under_lock`), `crates/kesh-reconciliation/src/mutex.rs`,
   `crates/kesh-api/tests/{bank_imports_e2e,bank_statements_manual_e2e}.rs`, `lib/features/bank-import/BankImportUpload.svelte`.
   **B** : `routes/(app)/supplier-invoices/+page.svelte` et son test, `routes/supplier_invoices.rs` (liste),
   `services/factures_fournisseur.rs`, `repositories/supplier_invoices.rs` (`list`), `routes/(app)/invoices/+page.svelte`
   (motif de pagination), `lib/features/fiscal-years/FiscalYearChecks.svelte`, `routes/(app)/+layout.svelte`, `src/app.css`,
   `routes/(app)/admin/mcp/+page.ts`, `tests/e2e/sidebar-navigation.spec.ts`, `lib/components/invoices/InvoiceForm.svelte` et
   son test, `routes/(app)/settings/vat-rates/+page.svelte`, `routes/vat.rs`, `routes/invoices.rs` (création, mise à jour), `lib/features/supplier-invoices/SupplierInvoiceForm.svelte`,
   `supplier-invoice-form.ts`, `SupplierInvoiceForm.test.ts`, `lib/features/invoices/invoice-helpers.ts` (`addDaysIso`),
   `lib/features/open-items/open-items.ts` (`todayLocal`), `lib/features/contacts/contacts.types.ts`,
   `routes/(app)/bank-accounts/bank-accounts-page.test.ts:702-718` (précédent `process.env.TZ`),
   `tests/e2e/supplier-invoice-draft-cycle.spec.ts`, `tests/e2e/helpers/inbox.ts`. **C** :
   `lib/features/reports/reports.api.ts` (`formatSwissDate`), les 32 fichiers du § 23 de la fiche, leurs tests,
   `crates/kesh-api/src/qr_decode.rs`, `crates/kesh-qrbill/src/parser.rs`, `crates/kesh-api/tests/inbox_import_e2e.rs`,
   `frontend/package.json`.
5. Le **manuel** : `docs/manual/fr/user-manual.tex` (`:95`, `:823`, `:833`, `:1014`, `:1032-1043`, `:1607-1612`,
   `:1888-1892`, `:2049-2053`) et `docs/manual/fr/admin-manual.tex` (tout l'inventaire du § *Manuel* de la fiche — `:111`,
   `:777-779`, `:2506`, `:2995-2998`, `:3886` — et toute phrase sur le jeton, le cookie, le renouvellement, la session).
   **Le PDF aussi**, lu aplati et vers la sortie standard seulement :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`). `docs/manual/{de,en,it}/` :
   confirme qu'il n'y a aucun `.tex`.
6. Pour contexte : `CLAUDE.md` du worktree (*Review Iteration Rule*, *Règle de splitting préventif*, *Issue Tracking Rule*,
   *Propagation post-patch*, *Recompter ses propres comptes rendus*, *Migration breaking policy*, *Pattern batch*) ;
   `docs/agents-orchestration.md` ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.

## Axes à exercer

**La remédiation de la P1 — axe obligatoire, à traiter en premier**

- Pour **chacun** des 22 findings de la P1 (rapport cité), le traitement écrit au Change Log (« Validation P1 (Sonnet) ») est-il
  **réellement** dans la fiche et le registre, et est-il **juste** ? Un finding déclaré appliqué mais absent, appliqué à moitié, ou
  dont le remède introduit un défaut neuf, est un finding de cette passe. *La sévérité se déplace vers ce qu'on vient d'écrire* :
  la conception de #876 a été **refaite** (renouvellement à l'entrée, pas `T`, gardes « session morte » et « 401 sans entrée »,
  échecs transitoires, désarmement, module inerte dans Vitest) ; relis-la comme un texte neuf.
- **#876 refait** : la borne d'inactivité annoncée (« entre `max(N, jwt) − T` et `max(N, jwt)` », « 14 à 15 minutes ») est-elle
  vraie au raisonnement, y compris quand `N > jwt`, `N < jwt`, `N = 1 min` ? La garde « 401 sans entrée » casse-t-elle un parcours
  légitime (une requête lancée par un clic **avant** que le gestionnaire d'entrée ait renouvelé ; l'ordre `pointerdown` / `click` ;
  un formulaire soumis au clavier ; un rechargement de page ; la première requête après `hydrate`) ? Le pas `T` et la retente à
  10 s peuvent-ils provoquer une boucle ou une rafale (plusieurs onglets, entrées rapides) ? Le coût (un renouvellement par minute
  d'activité) est-il acceptable avec la rotation et la détection de rejeu ? Les AC-A3 (a) à (j) et AC-A4 sont-ils décidables avec
  `vi.setSystemTime` seul ? Les mutations MA1 à MA10 tuent-elles ce qu'elles disent, et une régression probable reste-t-elle sans
  mutation ?
- **Les décomptes corrigés** (88 migrations, 18 / 33 / 8 entrées du menu, 64 / 51 / 31 / 20 lignes de l'AC-C3, 64 lignes de
  l'inventaire du manuel d'administration dont 54 hors classe et 10 au tableau, 16 lignes UTC, 27 choix) : recompte-les **depuis la
  source**, et greppe la valeur ancienne (`\b86\b`, `\b19\b`, `\b34\b`, « neuf entrées », « 1 à 22 ») dans la fiche et le registre :
  une occurrence hors d'une entrée datée du passé est un résidu.
- **Le découpage révisé** (A1, A2, B, C) : le tableau des fichiers est-il complet pour chaque sous-story (cherche les fichiers que
  les AC et les Tâches font toucher et que le tableau omet) ? A1 tient-elle en cinq modules ?

**La règle et sa preuve**

- **#876, la prémisse** : refais le raisonnement à la ligne près. Le cookie `kesh_access_token` a-t-il bien `max_age =
  jwt_expiry` ? Le jeton de renouvellement reçoit-il `now + refresh_inactivity` aux **quatre** sites (login, refresh,
  changement de mot de passe, assistant) ? Existe-t-il un chemin, quel qu'il soit, qui renouvelle **avant** le 401 (minuterie,
  `visibilitychange`, `hydrate`, garde de route, `+layout.ts`) ? `git grep` large : `refresh`, `expiresIn`, `setTimeout`,
  `setInterval`, `auth/refresh`, dans `frontend/src` (hors tests) — sortie copiée. La conclusion « le renouvellement sur 401
  échoue toujours avec 15/15 » est-elle exacte, ou une fenêtre (horloges, `leeway`, en-tête `Bearer`) la contredit-elle ?
- **#876, la conception** : `sessionExpiresIn` sur `/me` est-il calculable comme la fiche le dit (le cookie de renouvellement
  accompagne-t-il bien `/api/v1/auth/me`, chemin du cookie relu ; `/me` est-il derrière `require_auth` ; `find_active_by_token`
  rend-il ce qu'il faut) ? Horloge du poste fausse, serveur 0.17.1 sans le champ, renouvellement pendant une requête en vol, la
  déconnexion de tous les onglets : couverts ? Le verrou inter-onglets ferme-t-il la course de rotation **sur le chemin 401**
  aussi ?
- **#840** : les deux chemins (`create`, `create_csv`) sont-ils les **seuls** qui écrivent un relevé depuis un fichier (`git grep -n
  'create_import_or_roll_back\|create_with_transactions' crates/kesh-api/src`) ? Le déplacement « tel quel » sous le verrou
  préserve-t-il l'ordre des refus et les tests existants ? Le verrou du rapprochement pris par un import crée-t-il un risque de
  blocage (ordre des verrous : nommé puis lignes, deux connexions par import, épuisement du pool) ? Les tests AC-A7 sont-ils
  réalisables sur le modèle cité (helpers `take_account_lock`, `attendre_une_requete_en_cours`) ? MA11 à MA13 ? Les limites déclarées en Dev Notes (pool de 5, 409 du rapprochement) sont-elles exactes ?
- **#870** : le serveur fait-il vraiment ce que dit le § 10 (`total` filtré, écrêtage, liste de statuts) ? Le motif de
  `invoices/+page.svelte` est-il bien celui cité ? La lecture **réactive** de l'URL prescrite (`$app/state`, `$derived`, `$effect`, `goto`) est-elle réalisable sans boucle de navigation, et l'interaction case « annulées » / `?status=` complète (retour
  arrière du navigateur, `replaceState`, statut inconnu, `offset` au-delà du total) ?
- **#851** : la réfutation de l'hypothèse du rôle tient-elle (lis `auth.svelte.ts` en entier, `hooks.client.ts`, les gardes des
  pages `/admin/*`) ? La cause « débordement » est-elle établie au code (hauteur, `overflow`, nombre d'entrées, hauteur des liens
  en mode guidé et expert) ? L'oracle réécrit d'AC-B5 (sans défilement de la page, défilement de la barre seule) rougit-il vraiment avant correctif à 1280 × 720, et passe-t-il après ? Le correctif proposé suffit-il
  (barre `sticky`, flex, `min-h-0`) ?
- **#882** : le refus serveur (`verify_vat_rates_against_db`) est-il bien appelé à la mise à jour d'un brouillon ? Les **trois**
  sites qui posent `vatRate` dans `InvoiceForm` (`initLines`, `reloadInvoice`, `onProductSelect`) sont-ils couverts ? Le choix de
  ne pas changer le serveur (C-v0172b-9) est-il justifié ? Le message prescrit ne décrit-il que des actions qui existent ? La
  marque `_vatPlaceholder` (C-v0172b-27) est-elle retirée de **tout** envoi (`stripUiKey`, et tout autre chemin qui sérialise
  les lignes) ?
- **#885** : la prémisse déclarée fausse l'est-elle (`git grep -n 'defaultPaymentTermsDays\|default_payment_terms_days'`) ?
  `ContactResponse` porte-t-il `defaultPaymentTermsDays` dans la liste de fournisseurs passée au formulaire (pages `new` et
  `edit`) ? Les règles « suit la date », « saisie à la main », « édition » et #853 sont-elles décidables et complètes
  (fournisseur changé, date vidée, brouillon rouvert) ? `todayLocal()` fait-il ce que la fiche dit ? Le fuseau épinglé d'AC-B9 suffit-il (Node relit-il `process.env.TZ` à chaque appel) ? Les tests existants qui
  figent la date vide sont-ils tous trouvés (Vitest **et** E2E) ?
- **#880, inventaire clos** : rejoue toi-même le décompte des 32 fichiers et les deux motifs ; cherche **plus large** (dates
  passées en paramètre d'un message, `title=`, infobulles, exports CSV affichés, composants hors de ces dossiers **mais
  affichés sur ces écrans**). Un site manque-t-il ? Un site « conforme » ne l'est-il pas ? Les tests qui figent l'ISO sont-ils
  tous listés ? Le contrôle AC-C3 est-il décidable (et sa sonde positive) ? Les champs sont-ils des dates pures ou des
  horodatages (types TS et Rust) — `formatSwissDateTime` est-il employé là où il faut ? Rejoue la sonde de l'AC-C3 : 64 lignes avant filtre et 51 après sur `d7ec2ea2` ? La liste close des 20 lignes « après correction » est-elle exacte et complète ?
- **#883** : relis chaque point. .2 « sans objet » : vérifie l'absence de Prettier (`frontend/package.json`, racine). .5 : la
  fixture prescrite rend-elle bien `InvalidIban` avec la tolérance, et `InvalidSpcPayload` sous MC3 ? .4 : le comportement
  figé est-il celui du code ?
- **Le découpage** (C-v0172b-1) : est-il conforme à la *Règle de splitting préventif* (au plus cinq modules par sous-story, C
  mécanique) ? Un couplage entre sous-stories rend-il l'ordre A1 → A2 → B → C faux ?
- **Périmètre** : un défaut voisin non mécanique intégré à tort ? Un défaut mécanique renvoyé à tort ? Les issues demandées
  (C-v0172b-19, 20, 25) sont-elles justifiées ? Un point écarté sans issue (« on trace toujours ») ?
- **Les règles du dépôt** : aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 88 ; pas de P5, P6, P7) ; aucune
  donnée réelle ; aucun mot-clé de fermeture dans les commits prescrits ; gate complet et E2E complet au dernier commit de code
  de chaque sous-story ; captures en thème clair ; i18n quatre locales.

**Les textes et ce qu'ils affirment**

- **Le manuel** — axe obligatoire : chaque phrase prescrite au § *Manuel* est-elle **vraie au code après correctif** ?
  L'inventaire est-il clos : cherche dans le `.tex` **et le PDF aplati** toute phrase qui devient fausse ou muette (la session
  « prolongée à chaque action », le jeton de 15 minutes, « le lien ouvre la liste complète », « la date … est vide », la case des
  annulées, la TVA par ligne). Rejoue le relevé du manuel d'administration de la fiche (64 lignes) : le classement « hors classe » de chaque ligne tient-il, et chaque verdict du tableau est-il vrai au code (`SameSite`, stockage du jeton, durées) ?
- **CHANGELOG** : chaque puce est-elle vraie, bien rangée (*Corrigé* / *Ajouté*), sans fermeture de ticket ? L'intro « aucune
  migration, retour à la 0.17.1 sans restauration » tient-elle (champ JSON additif, `kesh_version_min_required`) ?
- **Les décomptes** (32 fichiers, 24 sites, 31 lignes de sites, 18 entrées d'Administration, 33 entrées du menu, 16 lignes UTC,
  27 choix, numéros de ligne) : recompte-les depuis la source (`git grep -c`, `sed -n`, `wc -l`) ; la ventilation est-elle
  cohérente avec son total ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0172b-validate-p2.md`. Il contient :

1. **Les findings**, numérotés `P1`, `P2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), la sous-story (A1, A2, B, C
   ou transverse), son origine (**remédiation P1** ou **défaut d'origine**), le lieu (`fichier:ligne` de la fiche, du registre ou du code), **la ligne lue citée** (sortie `grep -nF` ou
   extrait copié) qui fonde l'affirmation, et le remède proposé. Une affirmation de présence ou d'absence sans sortie `grep -nF`
   copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec, pour chaque axe exercé, ce que tu as lu ou cherché. Un « 0
   finding » sans cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **par sous-story** et **par origine**, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`, `tee`) —
  pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au rapport ; il ne se
  supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni
  `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh/`, `~/kesh-test/`, ni le NAS ; aucune donnée réelle dans le rapport.
- Ne relis le rapport de la P1 qu'**après** tes propres relevés : il sert l'axe de la remédiation, non de guide.
- Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture),
  `git log`, `git show`, `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh api` en `GET`.
