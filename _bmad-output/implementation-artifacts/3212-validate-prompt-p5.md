# Prompt — validation P5, Stories 32-12a et 32-12b (découpage de la 32-12)

*Versionné le 2026-10-24. Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, en contexte
frais, **Sonnet**. Lecture seule stricte. Elle suit la P4 (Haiku, ciblée, 0/0/2/1), dont les deux MEDIUM (P4-1, P4-2)
ont conduit l'orchestrateur à **retirer la dérogation et découper la fiche** (choix C-32-12-37). Son périmètre est **le
seul diff du découpage** — non la fiche entière.*

Dépôt `/home/gcorbaz/devel/kesh-32-12`, branche `story/32-12`. Fiche (unique, deux clés) :
`_bmad-output/implementation-artifacts/32-12-import-sauvegarde-anterieure.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3212.md` (C-32-12-37 neuf, C-32-12-36 annoté) ;
`_bmad-output/implementation-artifacts/sprint-status.yaml` (clés `32-12-import-sauvegarde-anterieure`,
`32-12a-regle-inventaire-et-textes`, `32-12b-bilan-ecran-manuel`). Rapport de la P4 :
`/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p4.md`. Règles : `CLAUDE.md` § *Règle de splitting préventif* et son
amendement D5, § *Review Iteration Rule* (une boucle ne se clôt que sur une passe sans finding au-dessus de LOW),
§ *Propagation post-patch*, § *Recompter ses propres comptes rendus* (« Recompter n'est pas réécrire l'historique »),
§ *Exception kesh-db*, § *E2E (Playwright)* ; `docs/agents-orchestration.md` § 2.1 et § 3.

## Le diff à relire — aplati, un seul

```sh
cd /home/gcorbaz/devel/kesh-32-12
C=$(git log -1 --format=%H -- _bmad-output/implementation-artifacts/3212-validate-prompt-p5.md)
git diff 797bbe0d "$C" -- _bmad-output/implementation-artifacts/ ':(exclude)_bmad-output/implementation-artifacts/3212-validate-prompt-p5.md'
```

`797bbe0d` est le commit d'avant le découpage ; `$C` celui qui le porte (et ce prompt). Les numéros de ligne d'un hunk
sont ceux du fichier **final** (`git show "$C":<fichier>`). Pour toute affirmation « X manque » ou « Y est encore là »,
vérifie par `grep -nF "<chaîne>" <fichier>` avant de l'écrire.

## Ce que tu cherches : ce que le découpage a cassé ou laissé entre deux clés

### Axe 1 — aucun AC ne tombe entre les clés (obligatoire)

Dresse, depuis la fiche finale, la table AC1 à AC21 → clé (32-12a, 32-12b ou les deux), à partir de **trois sources
indépendantes** : le tableau d'en-tête, la balise `[32-12…]` de chaque AC, et la § *Découpage* (*La coupe*) ; puis
confronte au registre C-32-12-37 et aux deux commentaires du `sprint-status.yaml`. Un AC sans clé, porté par deux clés
hors AC18/AC19, ou dont la clé diffère d'une source à l'autre — finding. Même contrôle pour les tâches (T0 à T7b) et les
mutations M1 à M9 (AC19a = M1-M4, M6-M8 ; AC19b = M5, M9) : chaque mutation est-elle tuée par un test d'un AC **de sa
propre clé** ? Chaque AC d'une clé qui **dépend** d'un livrable de l'autre (fixture de l'AC15, champ
`ParsedBackup::tables_absent` de l'AC5, AC8 et AC14 cités par l'AC10) le dit-il, et la dépendance va-t-elle bien de la
32-12b vers la 32-12a, jamais l'inverse ?

### Axe 2 — les décomptes de modules

Recompte, depuis les AC et les *Fichiers touchés* de chaque clé, les modules au grain de l'exemple de la règle
(`kesh-api/routes/invoices`) et **au même grain** que le total de huit de la story entière (fichiers sous `tests/` et
docs à part). La fiche annonce **cinq** pour la 32-12a (`kesh-db/backup`, `kesh-db/post_restore`,
`kesh-db/repositories`, `kesh-db/version`, `kesh-api/admin_backup`) et **trois** pour la 32-12b
(`kesh-api/routes/admin`, `kesh-i18n`, `frontend/admin-restore`). Un fichier qu'un AC d'une clé fait toucher sans qu'il
figure à ses *Fichiers touchés*, ou un module manquant au décompte — finding. Vérifie aussi, au code, la prémisse de
l'AC5 : l'implémentation de `Debug` de `ParsedBackup` est-elle bien **écrite à la main** (`crates/kesh-api/src/admin_backup/import.rs`,
`sed -n '85,110p'`) et aucun code de production hors `routes/admin.rs` ne lit-il le champ ?

### Axe 3 — l'ordre et l'état intermédiaire

L'ordre — 32-12a, puis 32-12b, les deux avant la 32-5 — est-il écrit partout où il doit l'être (en-tête, *Découpage*,
*Risques*, *Hors périmètre*, T0, registre, `sprint-status.yaml`) et sans contradiction ? L'affirmation que la 32-12a
ne rend faux **aucun** des trois sites du manuel (`docs/manual/fr/admin-manual.tex:2252-2263`, `:2273`, `:2370`) tient-elle
au texte du manuel (`sed -n '2240,2380p'`) ? Entre les deux fusions, `main` dirait-il quelque chose de faux (manuel,
`CLAUDE.md` P7 récrit par la 32-12a, CHANGELOG) ?

### Axe 4 — gates, CHANGELOG, comptes rendus

- AC18a / AC18b : la règle 🗄️ (`kesh-db` : gate complet même en boucle) s'applique-t-elle à la seule clé qui touche
  `kesh-db`, et l'autre suit-elle bien « gate ciblé en boucle, complet au dernier commit de code » ? L'E2E complet est-il
  justifié pour chacune (`frontend/tests/e2e/backup-restore-new-install.spec.ts` emprunte-t-il l'import ? `grep -n`) ?
- AC21 : une seule entrée de CHANGELOG, à la 32-12b — juste au regard de ce que livre la 32-12a seule ?
- Change Log : entrées « Validation P4 » (0/0/2/1, P4-1, P4-2, P4-3) et « Découpage » ; recompte leurs décomptes sur
  le rapport P4. Les entrées P1 à P3 du Change Log et les choix C-32-12-1 à 36 du registre sont-ils **intacts** (le diff
  ne doit y montrer aucune ligne retirée, hors l'annotation ajoutée sous C-32-12-36) ?
- `sprint-status.yaml` : `python3` t'est interdit ; vérifie l'indentation et la forme des trois lignes à l'œil
  (`grep -n "32-12" …`).
- **Propagation** : greppe les jetons de l'ancienne forme (`Dérogation`, `AC18\b`, `AC19\b`, `\bT4\b`, `\bT6\b`, `\bT7\b`,
  `Story 32-12\b`) dans la fiche : chaque occurrence restante est-elle une entrée datée (Change Log, relevé) ou une
  référence légitime ?

## Ce que tu rends

Findings `P5-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site, **preuve (commande et sortie
copiée)**, correction proposée. ⛔ **La liste des axes exercés ET non exercés** (axes 1 à 4) — sans elle, « 0 finding »
ne compte pas ; pour l'axe 1, la **table AC → clé** que tu as dressée ; pour l'axe 2, les **deux décomptes** que tu as
obtenus, module par module. Bilan par sévérité **recompté** sur la liste. Rapport écrit **d'un seul jet**, avec l'outil
d'écriture de fichier (non par une redirection du shell), dans **un seul fichier** :
`/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p5.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ **Aucune redirection d'écriture** (`>`, `>>`, `tee`, `2>` vers un fichier) — **pas même vers `/tmp` ou un répertoire
d'échange** : les deux lentilles de la P3 ont chacune laissé un fichier temporaire (`/tmp/x`, `…/scratchpad/ct.txt`),
c'est précisément ce qui est interdit. **Aucun fichier temporaire**, aucun fichier écrit hors de ton rapport ; pour
comparer deux sorties, utilise `diff <(…) <(…)`. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui
écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`,
`npm`, `npx`, `docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `unzip` vers le
disque (`unzip -l` et `unzip -p` vers la sortie standard seulement), `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`, `awk` en
lecture, `wc`, `diff <(…) <(…)`, `git log|show|diff|blame|ls-tree|tag --contains`, `gh issue view`, `pdftotext … -`,
`unzip -l|-p`.
