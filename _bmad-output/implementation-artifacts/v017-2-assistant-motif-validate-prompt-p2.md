# Prompt — validation P2 de la fiche v017-2 (release 0.17.0 : #809), deux lentilles, Opus

Tu valides la **fiche** de la Story v017-2 de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11).
**#809 (P3)** : l'assistant d'installation affichait un refus du serveur en message générique, sans son motif (constaté
le 2026-10-28 avec un pays saisi « Suisse » au lieu de `CH`) — un `catch` nu jetait l'erreur. La fiche retient : traiter
**l'assistant entier** (inventaire clos de ses `catch`, onze sites d'affichage dans `+page.svelte`), donner à
`errorMessageOf` un **repli optionnel**, corriger la branche `'VALIDATION'` morte de `reportFinalizeError`, ajouter une
**indication** au champ Pays (sans sélecteur ni `maxlength`), une garde lexicale, un test serveur du contrat de refus,
une spec E2E dédiée, le manuel utilisateur. Aucune migration. La validation P1 (Sonnet) a rendu 0 CRITICAL, 0 HIGH,
5 MEDIUM, 12 LOW, tous remédiés (Change Log, entrée « Validation P1 ») ; trois issues voisines ont été ouvertes (#817,
#818, #821).

Worktree `/home/gcorbaz/devel/kesh-0172`, branche `story/v017-2-assistant-motif`, partie de `c5101b07`. Langue du
rapport : **français**. Modèle : **Opus**. Passe **complète** (deuxième passe de validation). **Contexte frais** : ne
t'appuie ni sur les rapports P1 au-delà de ce que l'axe *Remédiation P1* demande, ni sur ce que la fiche dit
d'elle-même.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v017-2-assistant-motif.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v017-2-assistant-motif-choix-autonomes.md` (C-v017-2-1 à 13,
   annotations datées comprises).
3. Les issues, **en lecture seule** : `gh issue view 809 --json title,body,comments`, idem `644`, `710`, `97`, `538`,
   `44`, `817`, `818`, `821`.
3-bis. Les rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/v017-2-assistant-motif-validate-p1-R.md` et
   `…-p1-F.md` ; la remédiation : `git diff 6e127f8b HEAD -- _bmad-output/implementation-artifacts/`.
4. Le code : `frontend/src/routes/onboarding/` (`+page.svelte`, `+layout.ts`, `+layout.svelte`, `restore/`, les tests),
   `frontend/src/lib/features/onboarding/`, `frontend/src/lib/shared/utils/error-message.ts` (+ `.test.ts`),
   `api-error.ts`, `api-client.ts` (`parseErrorResponse`, `toFetchApiError`), `frontend/src/lib/shared/erreurs-en-texte.test.ts`,
   `i18n-literal-reader.js` ; côté serveur `crates/kesh-api/src/routes/onboarding.rs`, `address_input.rs`,
   `errors.rs` (bras `Validation`, `OnboardingStepAlreadyCompleted`), `crates/kesh-qrbill/src/validation.rs`
   (`validate_country`), `crates/kesh-api/tests/onboarding_path_b_e2e.rs` ; `frontend/tests/e2e/onboarding-path-b.spec.ts`,
   `docs/testing.md` § *Les échecs attendus* et l. 150-160 ; `frontend/src/lib/shared/e2e-selecteurs-traduits.test.ts` ;
   `frontend/tests/e2e/helpers/test-state.ts` (`authedApiContext`) ; `crates/kesh-seed/src/lib.rs:150-175` ;
   `frontend/src/lib/components/ui/input/input.svelte`.
5. Le **manuel** : `docs/manual/fr/user-manual.tex` l. 190-206, `docs/manual/fr/admin-manual.tex` l. 1069 et 1410-1422 ;
   les PDF lus aplatis, **vers la sortie standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
6. Pour contexte : `CLAUDE.md` du worktree (*Review Iteration Rule*, *Issue Tracking Rule*, *Recette visuelle*,
   *Test Locally First*) ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.

## Deux lentilles, en contexte frais chacune

L'orchestrateur lance **deux agents distincts** sur ce même prompt ; chacun ne tient **que** sa lentille.

### Axe commun et obligatoire aux deux lentilles — la remédiation P1

Pour **chaque** finding de la P1 (R1 à R7, F1 à F10), dans le périmètre de ta lentille (R : R1-R7 ; F : F1-F10) :
la remédiation écrite dans la fiche ou le registre traite-t-elle le finding, **sans en introduire un nouveau** ? Une
remédiation qui contredit une autre section non reprise (propagation), un numéro de ligne neuf faux, un texte i18n
neuf (`onboarding-language-error`) mal aligné sur ses voisines, un test ou une mutation neuve (M10, M11) qui ne rougit
pas pour la raison annoncée, une spec E2E dont un appel d'API ne correspond pas au corps attendu par la route
(`routes/onboarding.rs`, `onboarding.api.ts`) : chacun est un finding. Dis aussi si le rejet ou la reformulation d'un
finding P1 te paraît injustifié.

### Lentille R — la règle et sa preuve

- **Les prémisses 1 à 7** : chaque `fichier:ligne` cité par la fiche et le registre dit-il ce que la fiche lui fait
  dire ? La branche `'VALIDATION'` est-elle réellement morte (cherche tout producteur du code `VALIDATION`, serveur et
  client — `api-client.ts` compris) ? « Rien n'est écrit sur un refus » est-il vrai pour **chaque** étape, pas seulement
  les coordonnées (`seed_demo`, `start_production`, `set_accounting_language` qui charge un plan comptable) ?
- **L'inventaire clos** : la liste S1 à S11 et les deux sites déclarés couvrent-ils **tous** les `catch` et rejets de
  l'assistant ? Cherche les rejets non attrapés (`goto`, `loadI18nMessages`, `onMount`), les `.catch(`, les
  `{:catch}`, un composant importé par l'assistant qui attraperait lui-même.
- **AC-1** : le repli optionnel garde-t-il **à l'identique** le comportement des 23 fichiers appelants ? Le traitement
  d'un repli blanc est-il décidable et testé ? La garde `erreurs-en-texte.test.ts` (planchers, règles G1 à G7) reste-t-elle
  verte avec les liaisons ajoutées ?
- **AC-4** : la finalisation rejouée (`ONBOARDING_STEP_ALREADY_COMPLETED`) et le verrou `_loading` de `finalize`
  (`onboarding.svelte.ts:149-170`) interagissent-ils avec le nouveau chemin d'affichage ?
- **AC-5, la garde** : est-elle décidable telle qu'écrite (comment recueille-t-elle le bloc d'un `catch` ; que fait-elle
  d'un `catch` dans un balisage Svelte ou d'un commentaire) ? Son plancher et ses témoins suffisent-ils contre un recueil
  muet ?
- **Les tests prévus** : chaque site S1 à S11 est-il exercé, et le déclenchement prescrit (clic, soumission, gardes client
  de `submitCoordinates`) atteint-il bien l'appel d'API ? Le test serveur (AC-8) est-il rouge si la validation du pays
  disparaît ? La spec E2E est-elle jouable (sélecteurs, preset `fresh`, enchaînement des étapes réelles) ?
- **Les mutations M1 à M9** : chacune rougit-elle le test annoncé, et seulement lui ? Une régression probable reste-t-elle
  sans mutation ?
- **Le périmètre** : la fiche intègre-t-elle un défaut voisin non mécanique, ou renvoie-t-elle à une issue un défaut
  mécanique qu'elle aurait dû intégrer ?

### Lentille F — les textes, l'écran et ce qu'ils affirment

- **Le manuel (AC-9)** — axe obligatoire : la phrase prescrite (avec la mention du motif possiblement en français) est-elle vraie au code après la story (le motif s'affiche
  à **chaque** étape ? la valeur par défaut est-elle `CH` ?) ; une phrase actuelle du manuel utilisateur ou
  d'administration, au `.tex` **et** au PDF aplati, devient-elle fausse ou reste-t-elle muette sur ce qui change
  (assistant, coordonnées, pays, finalisation, comptes 1100/3000) ? La légende de la capture `onboarding-production.png`
  (`user-manual.tex:196`, image absente du dépôt) reste-t-elle juste ?
- **Le champ Pays (C-v017-2-4)** : l'argument contre `maxlength` (« Su » accepté) est-il vrai au code
  (`address_input.rs`, `to_uppercase`) ? Le choix « indication seule » répond-il à #809 point 2 ? L'alternative du
  sélecteur est-elle écartée pour de bonnes raisons ?
- **Les locales (AC-7)** : les quatre traductions sont-elles correctes et cohérentes avec `onboarding-field-ide-hint` ;
  la règle « repli égal à la valeur fr-CH » est-elle celle des gardes i18n du dépôt (lis-les) ? Le littéral de S1
  passe-t-il ces gardes ?
- **Le motif en français (C-v017-2-5)** : l'argument tient-il, et #710 couvre-t-il bien ces refus ?
- **CHANGELOG (AC-10)** : la phrase est-elle vraie et complète ? Les captures prescrites couvrent-elles chaque état
  visible touché (indication, toast de refus, finalisation) ?
- **Les règles du dépôt** : aucun mot-clé de fermeture prescrit dans un commit ; aucune donnée réelle ; aucune procédure
  sur la base du NAS ; « aucune migration » vrai ; les deux issues à ouvrir sont-elles justifiées et bien priorisées
  (P2 proposé pour le pays) ?
- **Les décomptes** de la fiche et du registre (onze sites, sept `catch` nus, treize lignes de la commande
  d'inventaire, 23 fichiers appelant `errorMessageOf`, deux clés neuves, M1 à M11, numéros de ligne) : recompte-les
  depuis la source.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, au chemin de ta lentille :
`/home/gcorbaz/devel/kesh-gate-logs/v017-2-assistant-motif-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/v017-2-assistant-motif-validate-p2-F.md` (lentille F). Il contient :

1. **Les findings**, numérotés `R1`, `R2`… ou `F1`, `F2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`),
   le lieu (`fichier:ligne` de la fiche, du registre ou du code), **la ligne lue citée** (sortie `grep -nF` ou extrait
   copié) qui fonde l'affirmation, et le remède proposé. Une affirmation de présence ou d'absence sans sortie `grep -nF`
   copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés** (l'axe *Remédiation P1* compris, finding P1 par finding P1) de ta lentille, un par un, avec, pour chaque axe exercé, ce que tu as
   lu ou cherché. Un « 0 finding » sans cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni
  `latexmk`, ni `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
