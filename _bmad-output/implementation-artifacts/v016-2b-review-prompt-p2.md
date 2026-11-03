# Prompt — revue de code P2 de la Story v016-2b (saisir un relevé à la main, l'écran) — passe ciblée, Opus

Tu relis **la seule remédiation de la revue de code P1** de la Story v016-2b de Kesh (comptabilité suisse, SvelteKit 5,
Rust, MariaDB) — saisir un relevé bancaire à la main, #816. Passe **ciblée**, **une lentille**, modèle **Opus**, contexte
frais : ne t'appuie sur aucune passe antérieure. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-0163`, branche `story/v016-2b-releve-manuel-ecran`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0163 diff 03c39b7f...HEAD` (`03c39b7f` = la fin du développement, relue en P1 ; la
remédiation est aux commits `fix(v016-2b) : revue P1 — …` et à l'entrée « Revue de code P1 » du Change Log). Les
rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/v0162b-review-p1-{B,E,A}.md`. Les décisions de l'orchestrateur et ce
qui a été appliqué ou non : l'entrée « Revue de code P1 » du Change Log de
`_bmad-output/implementation-artifacts/v016-2b-releve-manuel-ecran.md`.

*Le motif qui justifie cette passe : sur ce dépôt, la sévérité se déplace vers ce qu'on vient d'écrire. Ce qui reste à
relire n'est plus la story, c'est la remédiation.*

⛔ Ne lis rien sous `~/kesh-test/` (données réelles).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. Aucun fichier temporaire, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`). Un fichier créé par erreur se déclare, il ne se supprime pas.
- **Aucun `rm`**, ni `pkill`, ni `killall` ; **aucun `git checkout`/`git switch`/`git restore`**, aucun `git` qui écrit.
- **Rien qui compile ou exécute** : ni **`python3`**, ni `node`/**`npm`**/`npx`, ni **`cargo`**, ni **`make`**, ni
  **`docker`**, ni `mariadb`, ni **`curl`**, ni **`claude`**, ni **`sed -i`**, ni **`scripts/prepare-release.sh`**, ni
  aucun script du dépôt ou de `kesh-gate-logs`. **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`,
  `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`,
  `git show <réf>:<f>.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`. Sur le PDF aplati : `grep -cF`/`grep -oF … | wc -l`
  sous `timeout 60`, jamais de quantificateur `{0,N}` ; l'apostrophe y est `’`.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée.

## Les axes de la lentille

1. **La clé de fraîcheur** (`manual-statement.ts`, `manualSnapshotKey` ; `ManualStatementForm.svelte`, `currentKey`,
   `previewFresh`, les deux `$effect`). La table des rangs y entre-t-elle bien, de sorte que retirer ou ajouter une ligne,
   vide comprise, périme l'aperçu **et** le refus affiché ? Cocher une case périme-t-il encore quelque chose (ne doit
   pas) ? Le retrait de `seq` laisse-t-il une fenêtre où une réponse tardive serait acceptée (le formulaire est-il
   vraiment figé pendant l'aperçu — `fieldset disabled`, bouton « × », « Ajouter une ligne ») ? La sortie anticipée de
   `save` (succès : `submitting` reste vrai, `onSaved` hors du `try`) : un échec de navigation laisse-t-il l'écran figé
   pour toujours ? `previewStale` levé sur un refus de création qui demande une case : l'avis s'affiche-t-il ?
2. **Le rang affiché** : `screenRowsOf` est-elle la **seule** traduction d'une liste de rangs (aucun reste de
   `screenLines`, aucun `?? l - 1`, `git grep -nF "l - 1"`) ? Un rang inconnu : journalisé, texte générique (clé
   `bank-import-manual-warning-duplicate-lines-generic`), aucun rang brut à l'écran — y compris dans « Lignes concernées »
   d'un refus de création (`rows` vide → pas de phrase) ? Les tests M31 et M32 prouvent-ils ce qu'ils nomment (journaux
   `/home/gcorbaz/devel/kesh-gate-logs/v0162b/mutations/M31.log`, `M32.log`) ?
3. **Les jetons** : le formulaire n'emploie-t-il plus que des jetons définis dans `frontend/src/app.css`
   (`destructive`, `warning`, opacités) ? `git grep -nE "error-soft|warning-soft" -- 'frontend/src/*'` : quels emplois
   restent, et sont-ils tous antérieurs à la story (la tolérance de `jetons-non-definis.test.ts` reste-t-elle justifiée) ?
   Les captures reprises `/home/gcorbaz/devel/kesh-gate-logs/v0162b/captures/06-erreur-serveur-ligne-designee.png` et
   `12-avertissements-identique-doublons-chevauchement.png` montrent-elles la ligne désignée teintée et les cadres
   teintés ? Le bouton « × » reste-t-il hors de l'inventaire des boutons rouges ?
4. **Le manuel et l'écran sur la devise** : la puce « En CHF » du § *Saisir un relevé à la main* (`.tex` et PDF aplati)
   dit-elle ce que fait le code (`currency: "CHF"` en dur, aucun champ devise au compte) sans promettre de refus ?
   L'écran affiche-t-il « CHF » à côté des montants et dans l'introduction (quatre catalogues) ? « CHF » écrit en dur dans
   le balisage est-il admis par les gardes i18n ?
5. **La propagation des textes récrits** : `bank-import-manual-blocked-balances` (« sans séparateur de milliers »),
   `bank-import-manual-warning-duplicate-file` (« numéro »), `bank-import-manual-intro`, les deux clés neuves — repli et
   quatre catalogues égaux et traduits, `REPLIS_A_SITE_UNIQUE` (93, recompté), `sitesTotal` (2713, recompté par la
   commande du commentaire), parité.
6. **G24** (`crates/kesh-api/tests/textes_coherents.rs`) : l'extracteur part-il de la **déclaration** (ligne qui commence
   par l'ouverture), et rend-il une erreur sur toute ligne du bloc qui n'est pas une entrée ? Les nouveaux témoins
   couvrent-ils la mention en commentaire, la clé hors `[a-z_]` et la continuation ?
7. **La page** `/bank-import/manual` et son test (`bank-import-manual-page.test.ts`, le témoin
   `TemoinFormulaire.test-helper.svelte`) : le témoin peut-il être rendu hors des tests, ou gêner une garde qui parcourt
   les `.svelte` ? Le message d'échec du chargement.

## Rapport

Un seul fichier : `/home/gcorbaz/devel/kesh-gate-logs/v0162b-review-p2.md`, par l'outil d'écriture, jamais par une
redirection. Contenu :
1. **Les findings** : identifiant (`P1`, `P2`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et ligne, la
   preuve copiée, la correction proposée, et s'il est **né de la remédiation** ou antérieur.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise.
