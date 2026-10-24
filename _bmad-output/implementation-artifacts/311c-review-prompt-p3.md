# Prompt — revue de code P3 de la Story 31-1c, passe ciblée, une lentille (regression hunter), Sonnet

Tu relis **la seule remédiation** de la revue de code P2 de la **Story 31-1c** de Kesh (comptabilité suisse ; la
sauvegarde automatique faite par Kesh, issue #503). La P2 (Haiku, passe ciblée) a rendu **0/0/2/6** : M1 vérifié et
corrigé, M2 reclassé LOW par l'orchestrateur, trois LOW corrigés, quatre consignés pour une issue. Ce que tu cherches :
**les défauts que cette remédiation a introduits** — c'est là que se loge, mesuré sur ce projet, la sévérité d'une
boucle qui converge (`CLAUDE.md` § *La passe ciblée*). Langue du rapport : français. Modèle : **Sonnet**.

Worktree `/home/gcorbaz/devel/kesh-31-1c`, branche `story/31-1c`. Le diff à relire, **un seul diff aplati** :

```sh
git -C /home/gcorbaz/devel/kesh-31-1c diff 9c20768a..HEAD
```

Il porte un commit `fix(311c)` (code et tests : `crates/kesh-api/src/auto_backup.rs`,
`crates/kesh-api/tests/admin_auto_backup_e2e.rs`) et un commit `docs(311c)` (fiche, registre, ce prompt). Lis ensuite,
pour le contexte, et seulement elles :

- le rapport P2 `/home/gcorbaz/devel/kesh-gate-logs/311c-review-p2.md` (M1, M2, L1 à L6) ;
- au registre `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-311.md`, les choix **C-31-1-88 à
  C-31-1-90** et **C-31-1-93** ;
- dans la fiche `_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md`, l'entrée du Change Log « Revue de
  code P2 (ciblée, Haiku) de la 31-1c » ;
- les journaux de la remédiation, `/home/gcorbaz/devel/kesh-gate-logs/311c-p2r-*.log` (rouge, gate ciblé, gate
  complet, E2E) et `/home/gcorbaz/devel/kesh-gate-logs/311c-low-residuels.md` § *P2*. **Ne prends aucun chiffre
  déclaré pour argent comptant** : recoupe-le avec ces journaux.

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/311c-review-p3.md` — **le seul fichier que tu écris**.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucune redirection vers un
  fichier** (`>`, `>>`, `tee`, `-o`), pas même dans `kesh-gate-logs/` : la P2 a laissé par mégarde un `kesh-gate-logs/.noop` en redirigeant un `pdftotext`. Les filtres
  en lecture seule `tr`, `sort`, `cut`, `head`, `awk` (sans redirection) sont permis, **vers la sortie standard**.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/mem-guard.sh`, ni `scripts/regen-test-schema.sh`, ni `make`, ni `latexmk`, ni `cargo` (aucune
  sous-commande), ni `npm`, ni `npx`, ni `vitest`, ni `playwright`, ni `node`, ni `docker`, ni `mariadb`, ni `sqlx`, ni
  `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh` qui écrit ; `gh issue view 503` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `cat`, `ls`, `find`, les
  filtres ci-dessus, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` (vers la sortie standard).

## Exigences de preuve

- Tout finding porte sa **sévérité** (CRITICAL / HIGH / MEDIUM / LOW) et sa preuve copiée (`grep -nF` ou `sed -n`).
- ⛔ **Toute affirmation d'absence** (« le patch X n'a pas été appliqué », « aucun test ne couvre Y ») ou de présence
  d'un anti-pattern **s'adosse à un `grep -nF "<chaîne exacte>" <fichier>`** sur le fichier **final** (pas sur le
  diff), sortie copiée. Une affirmation d'absence sans cette preuve ne compte pas.
- Le rapport liste les **axes exercés** et les **axes non exercés**. Un « 0 finding » sans cette liste ne compte pas
  comme passe.

## Axes (la remédiation, et ce qu'elle a pu casser)

1. **Le contrôle de lisibilité de `auto/` dans `ecrire`** (C-31-1-93) : placé après `builder.create` et avant
   `supprimer_les_partiels`, il rend `Refusee(Failed)` si `read_dir` échoue. Peut-il faire échouer un cas qui devait
   réussir (dossier `auto/` neuf créé à l'instant, dossier lien vers un dossier lisible, dossier sur un partage réseau
   dont `read_dir` réussit) ? Le `ReadDir` ouvert est-il refermé avant l'écriture (descripteur gardé) ? Le statut
   `failed` (et non `disk_full` ou `insufficient_space`) est-il celui que l'état, le panneau et le bandeau affichent
   (`bannerKind`, `BACKUP_FAILURE_STATUSES` côté frontend) ? « Sauvegarder maintenant » (`POST …/run`) suit-il la même
   voie, et que rend-il alors ?
2. **Le manuel est-il vrai** : `docs/manual/fr/admin-manual.tex` § *Les sauvegardes du serveur* (« si le dossier auto/
   est illisible (remplacé par un fichier, droits modifiés), la liste montre encore les sauvegardes pré-import ; la
   sauvegarde automatique suivante échoue, et le panneau comme le bandeau le disent ») et le **PDF**
   `docs/manual/fr/admin-manual.pdf` aplati : chacune des trois formes (fichier, `0o600`, `0o300`) donne-t-elle
   maintenant cet échec ? Une autre forme de dossier « illisible » y échappe-t-elle encore (droits `0o100`, `0o200`,
   lien cassé) ? Le PDF n'a pas été régénéré (le `.tex` n'a pas changé) : vérifie que `git diff 9c20768a..HEAD --
   docs/` est vide et que le PDF porte la phrase.
3. **`ouvrir_la_sauvegarde`, `ENOTDIR` → `Ok(None)`** (P2-L2) : un `ENOTDIR` peut-il désigner autre chose qu'un
   dossier disparu (un chemin mal construit qui devrait rendre 500) ? Le test ajouté à `ouverture_sans_suivre_les_liens`
   exerce-t-il bien `ENOTDIR` (et non `ENOENT`) ? Le doc-comment (dernier composant, `auto/` lien suivi) dit-il le
   code ?
4. **Le test `liste_servie_malgre_un_dossier_auto_inutilisable`** : la branche (2) appelle désormais `run_once` sous
   `0o600`, la branche (3) sous `0o300`. Les droits sont-ils restaurés **avant** toute assertion qui peut paniquer ?
   L'assertion « rien d'écrit » (`noms_auto() == deposees`) peut-elle passer à vide (un `.partial` resté, un fichier au
   nom non conforme que `noms_auto` écarterait) ? Les `instant(1)`, `instant(2)` peuvent-ils rendre le test dépendant de
   l'horloge ? Le rouge déclaré (`311c-p2r-rouge.log`, `left: Done(Success(Ok))`) correspond-il bien à la branche (3) ?
5. **Doc-comments** : `supprimer_les_partiels` (raison de la suppression d'un `.partial` du futur, M2 reclassé),
   `appliquer_la_retention` (repli journalisé), `lister_les_sauvegardes`, `ecrire` — disent-ils le code, sans
   contredire un autre doc-comment du même fichier (« deux processus sur un même dossier » contre « une seule
   instance ») ?
6. **Propagation** : le symptôme corrigé (« rétention silencieusement désactivée », « 0o300 », « illisible ») se
   retrouve-t-il ailleurs — fiche, registre, manuel, CHANGELOG, i18n ×4 — sous une forme devenue fausse ?
   `git grep -nF` sur le worktree.
7. **Décomptes et journaux** : tests ajoutés (recompte `git show 9c20768a:<f> | grep -cE
   '#\[(test|tokio::test|sqlx::test)'` contre `HEAD`), chiffres du gate complet, de Vitest et de l'E2E au Change Log
   contre `311c-p2r-final*.log` ; échecs E2E tous dans `docs/testing.md` § *Les échecs attendus* ?
8. **Fiche et registre** : l'entrée P2 du Change Log réécrit-elle une entrée passée (interdit, `CLAUDE.md` § *Recompter
   ses propres comptes rendus*) ? C-31-1-93 dit-il le code ? La migration est-elle intacte (`git diff 9c20768a..HEAD --
   crates/kesh-db/migrations/` vide) ?
9. **Messages de commit** : `git log 9c20768a..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
   doit être vide.
