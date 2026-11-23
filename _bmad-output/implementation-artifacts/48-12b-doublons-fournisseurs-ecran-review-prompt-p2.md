# Prompt — revue de code P2 de la Story 48-12b (#975, #985), passe complète, Sonnet

Tu relis la **Story 48-12b** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB), release **0.20.0**, Epic 48.
Elle porte l'écran de quatre situations d'une facture fournisseur :
- le refus de doublon ;
- la réponse perdue d'un enregistrement ;
- la complétion relue ;
- le repassage en brouillon d'une annulée en doublon.

C'est la **deuxième passe** de la revue de code. La première (P1, trois lentilles Opus) a rendu **0 CRITICAL, 0 HIGH, 1 MEDIUM,
13 LOW** ; tout a été remédié. Cette passe est **complète** : la remédiation touche l'écran, ses tests, la spec E2E, le manuel et
le catalogue italien. Modèle : **Sonnet**, contexte frais, un seul agent qui exerce les trois regards (B : le diff seul ; E : chaque
branche et chaque borne ; A : la fiche contre le livré). Langue du rapport : français.

L'écran est une **machine à états** décrite par deux tables de la fiche, qui font foi :
- **table A** : le formulaire de saisie (création, complétion, modification), états E1 à E9 ;
- **table B** : la fiche d'une facture annulée, états F1 à F6.

Les **relevés C** (écritures asynchrones de la saisie) et **D** (gestes qui effacent l'avis F4) les complètent. Guy tient ses
livres réels dans Kesh : aucune saisie ne doit se perdre, aucun second clic ne doit enregistrer deux fois, aucune écriture réussie
ne doit être annoncée en échec.

Worktree `/home/gcorbaz/devel/kesh-48-12`, branche `story/48-12-reponses-perdues-fournisseurs`.

## À lire d'abord : les commits de remédiation

- `git -C /home/gcorbaz/devel/kesh-48-12 show cf38f506f` — l'écran et ses tests :
  - le décor complet du test du gel : projet, « Retenir », « C'est la bonne date », deux lignes, plus de `continue` muet ;
  - le test D8 ;
  - `typeof … !== 'number'` aux deux sites, avec ses tests ;
  - l'E2E E9 ;
  - la remise à zéro de `notifyWarning` ;
  - le doc-comment de `invoiceId` ;
  - le titre du test D6 ;
  - la phrase du manuel sur l'avis F4.
- `git -C /home/gcorbaz/devel/kesh-48-12 show 49df1f985 -- crates/kesh-i18n/locales/it-CH/messages.ftl` — les quatre clés
  italiennes de la 48-12b passées au tutoiement, par propagation d'un finding de la 48-12a.
- Le commit de la fiche, qui suit : Dev Agent Record (*Remédiation de la revue P1*) et Change Log.

Les rapports de la P1 sont à lire en lecture seule : `/home/gcorbaz/devel/kesh-gate-logs/e4812/review-12b-p1-{B,E,A}.md`.

Puis le diff entier de la story, **aplati** : `git -C /home/gcorbaz/devel/kesh-48-12 diff d5b283d49 cf38f506f` (le commit
`49df1f985` de la 48-12a s'y intercale : ne relis de lui que le catalogue italien). Lis aussi :
- la fiche `_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md` en entier ;
- D7 de la fiche 48-12a (les trois textes partagés).

Ne prends pas le Dev Agent Record pour argent comptant. Recoupe-le avec les journaux hors dépôt (lecture seule), sous
`/home/gcorbaz/devel/kesh-gate-logs/e4812/dev/` :
- `mut-12b/*.log`, `mut-12b/resume.log`, `mut-12b/resume-p1.log` ;
- le script `mut-12b/muter.py` (lis-le, ne le lance pas) ;
- les journaux de gate nommés au Dev Agent Record ;
- les captures `captures/e4812b-*.png`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `<`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se
  **déclare**, il ne se supprime pas.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code.** Sont interdits :
  - `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ;
  - **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
    `scripts/mem-guard.sh`, et tout autre script du dépôt ou de `kesh-gate-logs` (`muter.py`, `reset-bases.sh`, `e2e-e4812.sh`
    compris) ;
  - `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis :
  - `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e` ;
  - `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`) ;
  - `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `tr` ;
  - la lecture d'une image (outil de lecture de fichier) ;
  - pour le contrôle de PDF, seule forme admise : `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` ;
  - pour le `.tex` aplati, sans redirection : `cat <f>.tex | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les huit axes obligatoires

Ce sont les huit axes du prompt de la P1 (`48-12b-doublons-fournisseurs-ecran-review-prompt-p1.md`, à côté de la fiche ; lis-le
pour leur détail). Chacun s'exerce à `cf38f506f` :

1. **AC et décisions de Guy** : tables A et B, D2, D2bis (écart de lettre déclaré : `onRefusal` passé sans document), règle du
   drapeau.
   - E3 : le formulaire ne doit plus entrer en E9 qu'avec un identifiant numérique, et la page ne doit poser `locked` qu'alors.
     Une branche reste-t-elle où la saisie se fige sans avis ni reprise ?
2. **Relevés A à D par le symptôme** :
   - C : refais-le (les `await`, `$effect`, `.then` et rappels).
   - D : refais-le sur la fiche, gestes de lecture compris (aperçu, téléchargement : D8).
3. **Rejeu de `POST`/`PUT /invoices`** : aucun fichier de la facture client au diff.
4. **Textes partagés** :
   - les avis lisent les clés du serveur ; replis égaux au catalogue fr-CH ;
   - l'italien au tutoiement (glossaire `docs/i18n-glossaire.md`) dans toutes les clés neuves de la 48-12b ;
   - compteurs (`sitesTotal`, `CLES_RELEVEES`, `CANDIDATES_ATTENDUES`) : les tests neufs en ont-ils ajouté sans recompte ?
5. **Mutations** :
   - B48 à B53 (`resume-p1.log`) : chacune tuée par le test visé, sans erreur de compilation (« no tests » absent) ?
   - B48 sort-il vraiment un contrôle du `fieldset` ?
   - Le décor du test du gel rend-il chaque espèce de la fiche, active avant l'envoi ?
   - Une garde primaire a-t-elle été affaiblie pour rendre un test possible ?
6. **Manuels** : `user-manual.tex`, chaque phrase neuve contre le code (dont « dans un encadré à part », et l'aperçu et le
   téléchargement qui laissent l'avis). Contrôle négatif et positif sur le `.tex` aplati **et** sur le PDF aplati ; le PDF
   versionné n'est pas régénéré : point d'intégration, non défaut.
7. **Factures réelles existantes** : rien à l'écran ne refuse une facture existante à la lecture ; aucune saisie n'est vidée.
8. **Captures** : regarde-les toutes (15). La remédiation ne change aucun contrôle visible (tests, doc-comment, manuel) :
   confirme qu'aucune capture neuve n'était due. Le bandeau « réessai automatique » est expliqué au Dev Agent Record et suivi
   par #1414 : l'explication est-elle juste (`connectionreset`, `fetchWithRetry`) ?

Et la fiche contre le livré (regard A) :
- AC-1 à AC-10 ;
- le tableau « Tests à écrire » ;
- les écarts déclarés (D2bis, A1, captures, dossier des journaux) ;
- le statut `review` en tête, aux Dev Notes et à `sprint-status.yaml` ;
- l'entrée P1 du Change Log : tendance, modèles, reclassements ;
- l'issue #1414 aux Dev Notes et au Dev Agent Record.

## Rapport

Écris `/home/gcorbaz/devel/kesh-gate-logs/e4812/review-12b-p2.md`. Il contient :
- les findings numérotés (`P2-1`…), chacun avec sa sévérité (CRITICAL / HIGH / MEDIUM / LOW), son lieu (`fichier:ligne`), la
  ligne lue copiée, la preuve et le remède proposé ;
- **la liste des huit axes, exercés ET non exercés**, un par un ;
- le décompte final, **recompté depuis la liste**.

Un « 0 finding » non adossé à cette liste ne compte pas comme passe.
