# Prompt — validation P4 **ciblée** de la spec, Story 29-4 (une saisie concurrente ne passe plus sur un compte qu'on clôt)

*Versionné le 2026-10-14. **Une seule lentille**, contexte frais, en lecture seule stricte, **Haiku** (D6, rétro 25 :
Haiku est réservé aux passes ciblées ; § « La passe ciblée » du `CLAUDE.md`). Périmètre : **le seul diff de la
remédiation P3**, non la story entière.*

Dépôt `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4`. Fiche :
`_bmad-output/implementation-artifacts/29-4-cloture-compte-concurrente.md` ; registre :
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-4.md`.

**Historique.** P1 (Sonnet) R 0/0/6/7, F 0/0/2/7 → P2 (Opus) R 0/1/3/6, F 0/1/3/7 → P3 (Sonnet) R 0/0/1/11, F 0/1/0/6
(le HIGH de F et le MEDIUM de R sont le **même** défaut : la sonde `INNODB_TRX` prescrite en P2 exige le privilège
`PROCESS`). Rapports : `/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p3-{R,F}.md`. La remédiation P3 est le commit
qui suit `ffad9a23` et précède ce prompt.

## Le diff à relire — et lui seul

```sh
cd /home/gcorbaz/devel/kesh-29-4
git log --oneline ffad9a23..HEAD
git diff ffad9a23 HEAD~1 -- _bmad-output/        # HEAD~1 = la remédiation P3 ; HEAD = ce prompt
```

Lis le diff **aplati** ci-dessus (un seul intervalle), puis, pour toute affirmation, **le fichier final** (`sed -n`,
`grep -nF`), jamais les numéros de ligne d'un hunk. L'entrée « Validation P3 » du Change Log de la fiche résume la
remédiation et les décisions de l'orchestrateur, **à évaluer pour leurs conséquences, non à rediscuter dans leur
principe** : (1) sonde `INNODB_TRX` remplacée par le helper existant `sonde_verrou_nowait` ; (2) points sur MariaDB 11.6+
renvoyés à #669 ; (3) les LOW traités tels que listés.

## La lentille — « la remédiation P3 introduit-elle le défaut suivant ? »

*La sévérité se déplace vers ce qu'on vient d'écrire* : en P2 comme en P3, le défaut le plus grave venait du correctif
précédent. Axes, **dans cet ordre** :

1. **La sonde retenue (AC3, T29a-6 / T29a-7 ; C-29-4-21 annoté) — axe principal.**
   - Fonctionne-t-elle **sans aucun privilège** ? Lire `sonde_verrou_nowait` (`crates/kesh-db/src/test_fixtures.rs`,
     `grep -nF "pub async fn sonde_verrou_nowait"`) : quelle connexion, quel mode (autocommit), quelle erreur attendue
     (`1205`), que fait-elle sur une autre erreur ? Lire les droits versionnés `scripts/mariadb-init/01-dev-grants.sql` :
     la sonde ne lit-elle que ce que l'utilisateur `kesh` peut lire sur une base `_sqlx_test…` ? La fiche ou le registre
     gardent-ils **une** exigence de privilège (`PROCESS`, `information_schema.INNODB_TRX`, `INNODB_LOCK_WAITS`,
     `PROCESSLIST` hors filtre `DB = DATABASE()`) dans une **prescription vivante** ? (Les mentions qui disent pourquoi on
     n'en veut pas ne sont pas des prescriptions.)
   - Observe-t-elle **bien la file voulue** ? Dans le montage de T29a-6 / T29a-7 tel que la fiche le décrit (l'écrivain
     tient la ligne du compte en partagé par son `INSERT` à clé étrangère ; la clôture `close_in_tx` demande l'exclusif
     par `lock_for_closure`), la sonde `SELECT id FROM bank_accounts WHERE id = ? LOCK IN SHARE MODE NOWAIT` :
     (a) réussit-elle avant la clôture (garde (1 bis)) ; (b) peut-elle rendre `false` pour une **autre** raison que la
     demande exclusive de la clôture en file (un autre détenteur, un verrou d'intervalle, une autre ligne) ; (c) le motif
     d'arrêt de la boucle (tâche de la clôture terminée sans attendre) est-il écrit ; (d) la sonde elle-même peut-elle
     **perturber** le test (laisser un verrou, s'insérer dans la file devant l'écrivain) ? Le constat expérimental écrit
     (base `kesh_294`, table jetable, `10.11.16`) dit-il exactement ce qu'il a établi et ce qu'il n'a pas établi (la
     redemande par **lecture** `LOCK IN SHARE MODE`, non par contrôle de clé étrangère) ? Le repli écrit
     (`reconciliation_e2e.rs:7546-7553`) est-il fidèle à ce précédent ?
   - La sonde est-elle bien **existante** (aucune fixture neuve) et la boucle **locale au test** ? Le décompte des
     modules (quatre de code pour la 29-4a) est-il de nouveau vrai — `kesh-db/src/test_fixtures.rs` ne doit figurer dans
     aucune prescription d'écriture.
2. **Le résidu du cycle à quatre réécrit (F2)** : critère volet 2, AC4, Dev Notes § *Cycles* (argument de poids et
   *Résidus*), angles morts, C-29-4-14 annoté. Le cycle `Q → E → I → M → Q` est-il cohérent avec les verrous au sol pour
   chaque `E` nommé (clôture, réouverture, `PATCH`, W8, W27) ? « À égalité » / « strictement plus légère » est-il dit de
   façon identique partout ? Une phrase ancienne (« strictement le plus léger de tout cycle ») survit-elle comme
   prescription ?
3. **Les contingences et préconditions ajoutées (F3, F5, R2)** : T29a-4 / T29a-9 (seconde contingence, montage
   `mark_onboarding_complete` — vérifier `routes/bank_accounts.rs`, `assert_onboarding_complete`, et
   `bank_accounts_closed_e2e.rs:162`) ; T29a-10 (`company_invoice_settings`, `test_fixtures.rs:157`).
4. **Les textes et citations ajoutés (R3, R6, R7, R8, R9, R10, R11, R12, F7, F6)** : chaque numéro de ligne neuf cité par
   le diff (`errors.rs:1247-1249`, `routes/bank_imports.rs:57-61`, `reconciliation_cancel.rs:320-324`,
   `invoice_settlements_write.rs:892-894`, `supplier_invoices.rs:1097`, `:1338`, `bank_imports.rs:516-522`,
   `payment_batches.rs:80`, `:231`, `:236`, `api-external.md:594`, `bank_accounts.rs:839`, `:871`) — exact au sol ? Le
   texte prescrit pour `api-external.md:594` est-il vrai après la 29-4b ? « Du même ordre que deux sites préexistants »
   est-il exact (`routes/reconciliation.rs:2166-2170`, `bank_imports.rs:439-446`) ? Le renvoi à #669 correspond-il à ce
   que #669 porte (`gh api repos/guycorbaz/kesh/issues/669/comments`, lecture seule) ? L'extraction `G` (a) qualifiée
   rend-elle bien **neuf** fonctions (avec (b)) ?
5. **Le manuel** (`docs/manual/fr/user-manual.tex`, § *Clore un compte bancaire*, et son PDF aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) : la remédiation P3 change-t-elle quoi que ce
   soit à ce que l'AC9 prescrit au manuel, ou au statut « aucun texte de manuel pour 11.6+ » ? Vérifier qu'aucune
   prescription neuve ne rend un texte de manuel faux.
6. **Recomptes** : les décomptes de l'entrée P3 du Change Log (tests neufs onze / cinq, mutations 24, choix 22, choix
   annotés C-29-4-6, -7, -14, -18, -21, modules quatre / quatre / sept, comptés largement six / sept) — **recompte depuis
   la source**, avec la commande ; et ⛔ vérifie qu'aucune **entrée passée** du Change Log ni aucune annotation datée
   antérieure n'a été réécrite par la remédiation (un relevé daté se corrige par une annotation, jamais en place) :
   `git diff ffad9a23 HEAD~1 -- _bmad-output/` ne doit retirer aucune ligne d'une entrée de P1 ou de P2.

## Exigences de preuve

- ⛔ **Toute affirmation sur le fichier final s'étaye par `grep -nF "<chaîne exacte>" <fichier>`** (flag `-F`
  obligatoire), ou `grep -nFA <N>` pour un bloc, ou `sed -n a,bp` — avec la commande et sa sortie dans le rapport. Un
  finding qui affirme qu'un texte est **absent** ou **encore présent** sans cette preuve ne compte pas. Ne jamais
  raisonner sur les numéros de ligne d'un hunk de diff.
- Toute affirmation sur InnoDB est dite **raisonnée** ou **documentée au dépôt** (avec le fichier) ; aucune n'est
  vérifiée en pratique (interdit d'exécuter).
- Sévérités : `CRITICAL` / `HIGH` / `MEDIUM` / `LOW`, avec l'**origine** de chaque finding (remédiation P3, ou
  antérieure — un défaut antérieur hors du diff se signale en une ligne, sans être compté).

## Lecture seule stricte

Autorisé : `git log`, `git diff`, `git show`, `grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext … -` vers la sortie
standard, `gh api` **en lecture**. **Interdit** : toute commande qui écrit, compile ou exécute — `git checkout`,
`git switch`, `git restore`, `git stash`, `git reset`, `git commit` sous **aucune** forme ; `cargo` (build, test, run,
clippy, fmt), `npm`, `make`, `latexmk` ; `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et
tout script de `scripts/` ; toute requête SQL, `docker`, `mariadb` ; `gh` en écriture (issue, pr, comment) ; `sed -i`,
`perl -pi`, redirection `>` ou `>>` vers un fichier du dépôt. **Seul fichier écrit** : le rapport ci-dessous.

## Rapport

Écrire dans `/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p4-ciblee.md` :

1. **Bilan** `C / H / M / L`, **recompté** depuis la liste des findings.
2. **Findings** : sévérité, où, preuve (`grep -nF` / `sed -n` et sortie), conséquence, origine, correction proposée.
3. **Vérifié et trouvé juste** : un échantillon, chaque point avec sa commande.
4. ⛔ **Axes exercés et non exercés** : pour chacun des six axes ci-dessus, « exercé » (avec ce qui a été lu) ou « non
   exercé » (avec la raison). Un « 0 finding » sans cette liste **ne compte pas comme passe**. Dire en particulier si
   l'axe 1 (sonde sans privilège, file observée) a été exercé **jusqu'au bout**, ou en partie.
5. Les commandes exécutées, et la déclaration qu'aucune n'a écrit, compilé ni exécuté quoi que ce soit hors du rapport.
