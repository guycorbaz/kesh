# Prompt — revue de code P2 de la Story 31-1c, passe ciblée, une lentille (regression hunter), Haiku

Tu relis **la seule remédiation** de la revue de code P1 de la **Story 31-1c** de Kesh (comptabilité suisse ; la
sauvegarde automatique faite par Kesh, issue #503). La P1 (Sonnet, lentilles B, E, A) a rendu **0 MEDIUM+** (B 7 LOW,
E 9 LOW, A 4 LOW) ; dix LOW ont été corrigés, les dix autres consignés pour une issue. Ce que tu cherches : **les
défauts que cette remédiation a introduits** — c'est là que se loge, mesuré sur ce projet, la sévérité d'une boucle qui
converge (`CLAUDE.md` § *La passe ciblée*). Langue du rapport : français. Modèle : **Haiku**.

Worktree `/home/gcorbaz/devel/kesh-31-1c`, branche `story/31-1c`. Le diff à relire, **un seul diff aplati** (pas la
suite des commits — l'indexation des lignes d'un diff multi-commit est un piège connu) :

```sh
git -C /home/gcorbaz/devel/kesh-31-1c diff b4936ba2..HEAD
```

Il porte deux commits : `fix(311c)` (code, tests, i18n ×4, repli Svelte, manuel `.tex` et PDF, CHANGELOG) et
`docs(311c)` (fiche, registre, ce prompt). Lis ensuite, pour le contexte, et seulement elles :

- les rapports P1 `/home/gcorbaz/devel/kesh-gate-logs/311c-review-p1-{B,E,A}.md` (findings L-B1, L-B2, L-B4, E-3,
  E-4, E-5, F-A1 à F-A4) ;
- au registre `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-311.md`, les choix **C-31-1-88 à
  C-31-1-92** ;
- dans la fiche `_bmad-output/implementation-artifacts/31-1-sauvegarde-complete.md`, l'entrée du Change Log « Revue de
  code P1 de la 31-1c » et la liste « Écarts à la fiche et aux consignes » du Dev Agent Record de la 31-1c ;
- les journaux de la remédiation, `/home/gcorbaz/devel/kesh-gate-logs/311c-p1r-*.log` (rouges, gates ciblés, gate
  complet, E2E). **Ne prends aucun chiffre déclaré pour argent comptant** : recoupe-le avec ces journaux.

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/311c-review-p2.md`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. Aucun fichier temporaire : compare
  par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/mem-guard.sh`, ni `scripts/regen-test-schema.sh`, ni `make`, ni `latexmk`, ni `cargo` (aucune
  sous-commande), ni `npm`, ni `npx`, ni `vitest`, ni `playwright`, ni `node`, ni `docker`, ni `mariadb`, ni `sqlx`, ni
  `python3`, ni `sed -i`, ni `perl`. Aucune commande `gh` qui écrit ; `gh issue view 503` et `gh issue view 708` sont
  permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.

## Exigences de preuve

- Tout finding porte sa **sévérité** (CRITICAL / HIGH / MEDIUM / LOW) et sa preuve copiée (`grep -nF` ou `sed -n`).
- ⛔ **Toute affirmation d'absence** (« le patch X n'a pas été appliqué », « aucun test ne couvre Y ») ou de présence
  d'un anti-pattern **s'adosse à un `grep -nF "<chaîne exacte>" <fichier>`** sur le fichier **final** (pas sur le
  diff), sortie copiée. Une affirmation d'absence sans cette preuve ne compte pas.
- Le rapport liste les **axes exercés** et les **axes non exercés**. Un « 0 finding » sans cette liste ne compte pas
  comme passe.

## Axes (la remédiation, et ce qu'elle a pu casser)

1. **`fichiers_reguliers` tolère désormais les entrées illisibles** (C-31-1-88) — elle sert **quatre** appelants :
   `lister_les_sauvegardes`, `supprimer_les_partiels`, `taille_des_documents`, `appliquer_la_retention`. Pour chacun :
   le nouveau comportement (entrée sautée au lieu d'une erreur) peut-il **supprimer** une sauvegarde qui devait rester,
   en garder une qui devait partir de façon nuisible, sous-estimer dangereusement le seuil d'espace, ou masquer un
   problème qu'un statut aurait dû dire ? La rétention « en garde plus, jamais moins » : vérifie-le sur le code.
2. **`lister_les_sauvegardes`** : seule l'erreur du dossier `auto` est tolérée, celle du dossier de sauvegarde reste
   `Err` (500). Le `match` est-il juste (garde `if kind == BackupKind::Auto`) ? Le test
   `liste_servie_malgre_un_dossier_auto_inutilisable` exerce-t-il les deux branches annoncées (`read_dir` en échec,
   métadonnées en échec sous `0o600`) et restaure-t-il les droits **avant** toute assertion qui peut paniquer ? Un test
   lancé en root le rendrait-il muet (le `0o600` ne bloque pas root) ?
3. **`ouvrir_la_sauvegarde`** (C-31-1-89) : `rustix::fs::open` en `RDONLY | NOFOLLOW | CLOEXEC`, `NOENT` et `LOOP` →
   `Ok(None)`, `fstat` non régulier → `Ok(None)`. `O_NOFOLLOW` ne porte que sur le **dernier** composant : est-ce
   cohérent avec la liste (qui ne suit pas non plus les liens d'`auto/` lui-même ? — vérifie ce que fait `read_dir` sur
   un `auto` qui est un lien) ? Ouvrir une FIFO sans `O_NONBLOCK` bloque-t-il le fil `spawn_blocking` ? Le handler
   `download_backup` : ordre des `map_err`, `ok_or_else(introuvable)`, audit émis seulement après l'ouverture réussie ?
   `Content-Length` (taille de la liste) contre le fichier réellement ouvert.
4. **`supprimer_les_partiels`** (C-31-1-90) : écart absolu `>= 10 min`. Un `.partial` **en cours d'écriture** par ce
   processus ou un autre peut-il désormais être supprimé (horloges, granularité des dates de fichier) ? Le test
   `partiel_date_du_futur` couvre-t-il une heure dans le futur (supprimé), une minute dans le futur et dans le passé
   (gardés), et ne dépend-il pas de l'ordre de `read_dir` ?
5. **`DELAI_DE_REPRISE`** (C-31-1-91) : `is_due` et `next_due_at` disent-ils toujours une seule règle ? Les tests
   `is_due_echec_*` et `next_due_at_meme_regle_qu_is_due` restent-ils significatifs ? Reste-t-il une mention de
   `delai_de_reprise` ou de `min(intervalle, 1 h)` dans le **code** ou ses doc-comments (`git grep -nF`) ? (Les
   entrées datées du registre C-31-1-17 et C-31-1-85 n'ont pas à changer : ce sont des relevés.)
6. **Textes** — `admin-backup-auto-reminder` dans les **quatre** `messages.ftl` et le repli de
   `AutoBackupSection.svelte` : le repli est-il **identique au caractère près** à la valeur fr-CH (compare par
   `grep -F` d'une sous-chaîne longue sur les deux fichiers) ? Les traductions disent-elles la même chose que le fr-CH
   (nombre conservé plus une ; une par import, jamais purgée) ? Une ponctuation française (espace avant `:`) est-elle
   respectée en fr-CH ?
7. **Manuel** `docs/manual/fr/admin-manual.tex` **et PDF** `docs/manual/fr/admin-manual.pdf` (aplati) : les quatre
   phrases neuves ou reformulées (reprise « au bout d'une heure », « au moins deux intervalles », liste servie malgré un
   `auto/` illisible, `.partial` daté du futur) disent-elles ce que fait le code ? « le panneau comme le bandeau le
   disent » : le bandeau s'allume-t-il vraiment sur `failed` (activée seulement) ? Le PDF porte-t-il ces phrases (il a
   été régénéré : vérifie qu'il n'est pas périmé par rapport au `.tex`) ? `CHANGELOG.md` : « au moins deux
   intervalles ».
8. **Garde G22** (`crates/kesh-api/tests/textes_coherents.rs`, `la_sauvegarde_automatique_dit_vrai`) : ses motifs
   négatifs peuvent-ils passer à vide (normalisation de `\textbf`, des `~`, de l'apostrophe) ? Ses positifs sont-ils
   adossés au bon fichier ? Le tableau des gardes en tête de fichier porte-t-il G22 ?
9. **Fiche et registre** : l'annotation (q) et #708, les écarts ajoutés (AC32 ×2, AC25/AC28, AC23 `TINYINT
   UNSIGNED`) sont-ils exacts **contre le code et la migration** (`sed -n` de
   `crates/kesh-db/migrations/20261024000001_backup_schedule.sql`) ? Une entrée datée d'une passe passée a-t-elle été
   réécrite (interdit : `CLAUDE.md` § *Recompter ses propres comptes rendus*) ? La migration est-elle **intacte**
   (`git diff b4936ba2..HEAD -- crates/kesh-db/migrations/` doit être vide — P8) ?
10. **Décomptes déclarés** au Change Log (tests ajoutés, chiffres des gates, échecs E2E nommés) : recompte-les
    (`git show b4936ba2:<f> | grep -cE '#\[(test|tokio::test|sqlx::test)'` contre `HEAD`) et compare-les aux journaux
    `311c-p1r-*.log`. Les échecs E2E sont-ils tous dans `docs/testing.md` § *Les échecs attendus* ?
11. **Messages de commit** : `git log b4936ba2..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
    doit être vide.
