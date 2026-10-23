# Prompt — revue de code P2 **ciblée** de la Story 31-1b, une lentille T, Haiku

Tu relis **une seule remédiation** de la Story 31-1b de Kesh (restaurer une installation neuve depuis l'assistant,
issue #503) : le commit qui traite la revue de code P1. Selon `CLAUDE.md` § *La passe ciblée*, la sévérité se déplace
vers ce qu'on vient d'écrire : ta lentille ne relit **que ce diff**, pour y chercher les défauts qu'il **introduit**.
Worktree `/home/gcorbaz/devel/kesh-31-1b`, branche `story/31-1b`. Le diff, aplati :
`git -C /home/gcorbaz/devel/kesh-31-1b diff 1fcb7732 ea51180b` (deux commits : `9d6f3184`, la remédiation, et `ea51180b`, un mot allemand corrigé après le gate). Langue du rapport : français. Modèle : **Haiku**.

Contexte à lire : dans `_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md`, l'entrée du Change Log
« Revue de code P1 de la 31-1b » (ce qui a été remédié, consigné, et pourquoi) ; au registre
`epic-31-choix-autonomes-311.md`, C-31-1-78 à C-31-1-80 ; les rapports P1
`/home/gcorbaz/devel/kesh-gate-logs/311b-review-p1-{B,E,A}.md`. Ne refais pas la revue P1 : les findings **consignés**
(B-4, B-5, B-6, E-LOW-1, E-LOW-4, E-LOW-6, A-L6) ne sont pas des findings de cette passe.

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/311b-review-p2-T.md` — bilan CRITICAL / HIGH / MEDIUM / LOW ; chaque
finding avec sa preuve **copiée** (sortie `grep -nF` ou `sed -n`) ; la liste des **axes exercés** et celle des **axes non
exercés**. Un « 0 finding » sans cette liste ne compte pas.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport ; aucun fichier temporaire.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/mem-guard.sh`, ni `make`, ni `latexmk`, ni `cargo`, ni `npm`, ni `npx`, ni `node`, ni `docker`, ni
  `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh` qui écrit.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ Pour tout finding qui affirme qu'un code **manque** ou qu'un anti-motif **reste**, la preuve est un `grep -nF` sur
  le fichier **à `HEAD`**, jamais une ligne lue dans le diff (`CLAUDE.md` § *Haiku-specific guardrails*).

## Lentille T — la remédiation et ce qu'elle a pu casser

1. **Textes (quatre locales et replis)** : `admin-backup-files-not-included`, `admin-restore-files-missing`,
   `admin-restore-confirm-body` dans `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` et les replis de
   `AdminBackupPanel.svelte` / `AdminRestorePanel.svelte` : replis **identiques** au fr-CH au caractère près ; variable
   `{ $count }` présente dans chaque traduction ; plus aucun « document indisponible » ni ses traductions (`git grep -n`
   des quatre formes à `HEAD`). Les gardes qui figent des textes (`crates/kesh-api/tests/textes_coherents.rs`,
   `frontend/src/lib/shared/i18n-keys.test.ts`) sont-elles concernées ?
2. **Manuel** : `docs/manual/fr/admin-manual.tex` **et** `admin-manual.pdf` (aplati). Chaque citation entre « » ajoutée
   ou modifiée par le diff est-elle le texte **réel** d'un écran ou d'un message serveur (catalogue fr-CH,
   `error-source-document-gone` / `imported-supplier-invoices-doc-gone`, `error-invoice-pdf-gone`) ? Les affirmations
   neuves tiennent-elles au code : « le bouton d'import est alors désactivé », « rechargez la page » pour réessayer, la
   condition de `force_onboarding_done_if_eligible` (`crates/kesh-db/src/backup.rs`) ?
3. **`AdminRestorePanel.svelte`** : `unknownOutcome` désactive le fichier et « Importer », et `requestImport` le refuse ;
   est-il remis à `false` quelque part qui le rendrait inopérant, ou bloque-t-il un cas qu'il ne devait pas (erreur
   ordinaire 400/409/413/500, succès) ? « Se reconnecter » reste-t-il atteignable ?
4. **`routes/onboarding/restore/+page.ts`** : `export const load: PageLoad` avec `await parent()` — `parent()` du
   layout `routes/onboarding/+layout.ts` a-t-il bien chargé `onboardingState` avant le contrôle d'étape (le layout
   peut-il échouer en silence et laisser `loaded` faux, donc la garde d'étape inopérante — est-ce acceptable) ? La garde
   de rôle est-elle toujours évaluée avant tout rendu ? L'ordre « rôle puis étape » change-t-il une redirection ?
5. **Tests** : `AdminRestorePanel.test.ts` (trois tests neufs), `onboarding-restore-page.test.ts` (réécrit : la
   fonction `charger` appelle-t-elle vraiment le `load` exporté ; l'état d'onboarding d'un test fuit-il dans le
   suivant ?), `backup-restore-new-install.spec.ts` (assertions exactes : correspondent-elles au texte rendu, point final
   compris ?).
6. **Mutations M-b11 à M-b13** : le diff de chaque journal `/home/gcorbaz/devel/kesh-gate-logs/311b-mut-M-b1{1,2,3}.log`
   est-il la mutation décrite au Change Log, et l'échec porte-t-il sur l'assertion qui compte ?
7. **Comptes rendus** : l'annotation datée des hashes après report (correspondance ancien → nouveau, à vérifier par
   `git log --format='%h %s' origin/main..HEAD`) et le recompte « 11 clés neuves » sont-ils justes ? Les gates déclarés
   pour `9d6f3184` et `ea51180b` correspondent-ils aux journaux `311b-*-5.log` et `311b-gate-6.log` ?
