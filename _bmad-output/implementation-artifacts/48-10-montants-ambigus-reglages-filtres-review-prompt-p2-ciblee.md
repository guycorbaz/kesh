# Prompt — revue P2 ciblée de la Story 48-10 (#1173, #1175, #1176, #1177) : une lentille, le seul commit de remédiation

Tu relis la **remédiation des LOW de la revue P1** de la Story 48-10 de Kesh (comptabilité suisse, SvelteKit 5 runes),
release **0.20.0**, Epic 48. La P1 (trois lentilles Opus) a rendu 0 CRITICAL / 0 HIGH / 0 MEDIUM / 14 LOW
(`/home/gcorbaz/devel/kesh-gate-logs/e4810/review-p1-{B,E,A}.md`). Les LOW qui touchent la production ont été corrigés
dans la story : le commit de remédiation **`531397f5a`**, suivi de **`d1afc9384`**, qui ne relève que le compteur
`sitesTotal` de `i18n-keys.test.ts` (3496 → 3502, révélé par `test:unit` après le premier commit). Ta passe porte sur
**ces deux commits seuls**, aplatis : `git -C /home/gcorbaz/devel/kesh-48-10 diff 531397f5a~1 d1afc9384` (et `--stat`). Le
commit qui les suit ne porte que la fiche, le registre et ce prompt. Vérifie aussi le compte : six sites neufs, tous de clés
existantes (`nonPriceMessages()`, `onFeeInput()`) ?

Une seule lentille : **exactitude de la remédiation, et ce qu'elle a pu casser** (la sévérité se déplace vers ce qu'on
vient d'écrire). Modèle : **Sonnet ou Haiku** (la P1 était Opus). Contexte frais. Langue du rapport : français.

Lis pour le contexte : les trois rapports P1 ; la fiche `_bmad-output/implementation-artifacts/48-10-montants-ambigus-reglages-filtres.md`
(D1 à D6, AC1 à AC12, Dev Agent Record, § *Remédiation P1*) ; le registre `e0200-choix-autonomes-e4810.md` (C-e4810-6 et 7).
Journaux (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e4810/` : `remed-p1-vitest-2.log`, `remed-p1-check-pre.log`,
`mut/resume.log` (section « remédiation P1 ») et `mut/M19-litteral.log`, `M21.log` à `M26.log`, `M17u.log`,
`bases-remed-p1.log`, `remed-p1-test-fast.log`, `remed-p1-front.log` (un rouge, `sitesTotal`), `remed-p1-front-2.log`,
`remed-p1-e2e.log`. Captures (thème clair) sous
`captures/` : `e4810-minimum-ambigu.png` (bouton *Enregistrer* estompé), `e4810-filtre-ambigu.png`,
`e4810-filtre-illisible.png`, `e4810-filtre-signet-ambigu.png` (liste non vide), `e4810-rappel-frais-ambigu.png` (message
sous le champ *Frais*) — regarde-les si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`** ; ni `pkill`, `killall`, `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`**, aucun `git` qui écrit.
- **Rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (`e2e-e4810.sh`, `mut/mutations-e4810*.py`) ; ni
  `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git show`, `git diff`, `git log`, `git grep`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` (seule forme du contrôle de PDF).
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Axes obligatoires

1. **Article — effacement du pied (B1, E1 ; C-e4810-7)** : `products/+page.svelte`, `nonPriceMessages()` et
   `onPriceInput()`. La liste fermée nomme-t-elle exactement les messages de `formValidation` (nom) et de `submitForm` (doublon,
   TVA) — mêmes clés, mêmes replis ? Un message qui ne vient pas du prix peut-il encore être effacé à tort (un refus du serveur
   sur la description, le compte, la TVA) ? Un message du prix peut-il rester ? Le doc-comment dit-il vrai ? Les tests M24 et
   « doublon de nom » ; `entree()` passée de `requestSubmit()` à `fireEvent.submit` — les tests « Entrée » d'avant prouvent-ils
   davantage ou moins ?
2. **Rappels (B2, B3)** : `settings/dunning/+page.svelte` — `onFeeInput` (n'efface pas le refus du délai ; efface le reste) ;
   le `<p id="{uid}-fee-error">` dans le bloc du champ *Frais* (mise en page à la largeur `max-w-xs`, `aria-describedby`
   inchangé) ; plus aucun `dunning-form-fee-error` hors de ce bloc.
3. **Lecture au centime (B4, B5 ; C-e4810-6)** : `features/journal-entries/amount-entry-centimes.ts` neuf —
   `round(2, 0).eq(value)` ; la charge ramenée par `toFixed(2)` seulement quand la saisie écrit plus de deux décimales ; une forme
   que le serveur accepte (frais : `scale_within(fee, 2)` sans `normalize`, `crates/kesh-api/src/routes/dunning_levels.rs` ;
   minimum : `scale_within(&min.normalize(), 2)`, `company_invoice_settings.rs`) devient-elle refusée ? Un montant **enregistré**
   (frais `DECIMAL(7,2)`, minimum rendu à quatre décimales, `etape0-minimum.log`) rouvert change-t-il de **valeur** ? Le minimum
   part désormais `5.00` au lieu de `5.0000` : même valeur ? Un seul prédicat (`minimumProblem`) au message, au bouton et à la
   garde de `save()` ? `balance.ts` intact (D6, AC7) ?
4. **Tests** : `amount-entry-centimes.test.ts` (neuf), `dunning-page.test.ts` (M21, M22, M23, « 10.0100 »),
   `settings-invoicing-page.test.ts` (charges au centime, comparaison en valeur, bouton sur `12'34`, M17u),
   `products-page.test.ts` (B1, E1, doublon, E2/M25), `journal-entries-list-page.test.ts` (« ␣1.250␣ »). Un test passe-t-il
   **à vide** ? Les mutations `M19-litteral`, `M21` à `M26`, `M17u` (`mut/resume.log`) sont-elles tuées pour la bonne raison ?
5. **E2E** (`frontend/tests/e2e/montants-ambigus.spec.ts`) : aide `capture` en `animations: 'disabled'` ; le test du filtre sur
   deux écritures créées par l'API (`createJournalEntryViaApi`, 100.00 et 2000.00) — « 1'250 » n'en montre qu'une, « 1.250 »,
   « 12'34 » et le signet les montrent toutes deux ; capture `filtre-illisible`. Le journal `remed-p1-e2e.log` porte-t-il
   l'en-tête du commit, et ses captures sont-elles postérieures au commit ?
6. **Manuels contre le code, PDF compris** : `docs/manual/fr/admin-manual.tex` item *Frais* (`:1348` : « le message s'affiche
   sous le champ », « Une saisie à exposant … ou précédée d'un signe n'est pas lue ») et § *Montant minimum d'une facture*
   (`:3911` : « deux décimales au plus ») ; `docs/manual/fr/user-manual.tex` § *Le montant minimum d'une facture* (`:1495`).
   Sont-ils vrais au code remédié (« 0.125 » refusé au client, « 10.0100 » accepté) ? Un manuel doit-il désormais dire que
   « 10.125 » est illisible ? PDF non régénérés dans la story (intégration) — vérifie qu'ils ne portent pas encore les phrases
   neuves : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'refusé comme ambigu'`
   (attendu 0) ; nomme-le comme point d'intégration.
7. **Propagation** : greper le symptôme corrigé sur le dépôt — `git grep -n "readJournalAmount(fFee\|readJournalAmount(minimumAmount\|requestSubmit" -- frontend/src`
   — et les LOW de la P1 **non** corrigés dans ce commit (captures, journaux : A1, A2, A5, A6 sont de la preuve, refaite ; A3,
   E3 portés par l'E2E) : chacun est-il traité ou dit ?

## Rapport

Un fichier : `/home/gcorbaz/devel/kesh-gate-logs/e4810/review-p2-ciblee.md`. Findings numérotés (`C1`, `C2`…) avec sévérité
(CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des
axes exercés ET non exercés** ; le décompte final recompté depuis la liste. Un « 0 finding » sans cette liste ne compte pas.
