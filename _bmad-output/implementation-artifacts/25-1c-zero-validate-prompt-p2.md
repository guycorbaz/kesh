# Prompt — passe 2 de `bmad-create-story validate`, Story 25-1c-zero

*Versionné le 2026-09-15. Une lentille en contexte frais (Opus), orthogonale aux deux lentilles de
la passe 1 (Sonnet, Haiku).*

Tu es un **valideur adversarial** en contexte frais. Ton objet est le fichier
`_bmad-output/implementation-artifacts/25-1c-zero-audit-company-id.md`, dépôt
`/home/gcorbaz/devel/kesh`. Ta mission n'est pas d'approuver : c'est de **trouver ce qui ferait
échouer, dévier ou mentir** l'implémentation qui suivra.

⛔ **Rien ne se croit sur parole** — ni la spec, ni son Change Log, ni les conclusions de la passe 1
qu'il rapporte. Chaque affirmation se **vérifie dans le code**.

⚠️ **Ne conteste PAS les décisions arbitrées** — le mécanisme (sous-SELECT, `BIGINT NULL` sans FK,
ni `COALESCE` ni rejeu post-restore) et les deux arbitrages du 2026-09-15 (aucune release avant le
merge ; la story mesure les deux sous-cas du restore, la 25-1c tranche). Conteste leur **mise en
œuvre**.

## Ce que la passe 1 a changé, et pourquoi c'est là qu'il faut regarder d'abord

Le motif mesuré du dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire.* La passe 1 a
corrigé un **total faux** (« 89 sites » → 106, et exigence d'un commentaire sans total), une
**liste de faux positifs supposée** (régénérée depuis l'exécution d'un grep), et deux dérives de
ligne. `git diff 08187010 -- _bmad-output/` montre exactement ce qu'elle a touché — **c'est ta base
de comparaison, et tu la nommes dans ton rapport**.

## Sources

- l'epic `_bmad-output/planning-artifacts/epic-25-vague1-suite.md`, la story sœur
  `25-1a-piste-inalterable.md`, l'issue **#378** ;
- `CLAUDE.md` § *Migration breaking policy* (P1 à P8), § *Recompter ses propres comptes rendus* ;
- **le code**, qui prime.

## Les axes, et tu déclareras lesquels tu as exercés

1. **La remédiation de la passe 1.** Les nombres qu'elle a écrits (106, 38, 110, 16, 8 → 6 lignes
   de grep) sont-ils **recomptés** par toi, commande à l'appui ? Le résultat attendu des deux greps
   de l'AC 17 est-il exactement celui qu'on obtient ? A-t-elle laissé le symptôme ailleurs dans la
   spec, l'epic ou `sprint-status.yaml` ?
2. **Tout nombre de la spec, recompté depuis la source** — pas relu. Chaque nombre porte-t-il son
   périmètre ? Deux grandeurs différentes portent-elles le même nombre ?
3. **Faisabilité de la migration et de l'alimentation.** Relis l'AC 1 à 5 comme le développeur qui
   les exécutera. Tu peux **vérifier la syntaxe SQL dans une transaction annulée** sur la base de
   dev (`docker exec kesh-mariadb-dev mariadb -uroot -pkesh_dev_root`), **uniquement sur une base
   jetable que tu crées et supprimes toi-même** (`CREATE DATABASE _p2_scratch` … `DROP DATABASE
   _p2_scratch`) — jamais sur `kesh` ni `kesh_e2e`.
4. **Les tests et les mutations (AC 7, 8, 14, 15, 16).** Pour chacun : le montage est-il
   constructible avec les helpers existants — cite-les —, et l'assertion **tranche-t-elle** ? Pour
   l'AC 8, construis mentalement les deux sous-cas pas à pas : où l'`id` de la société restaurée
   est-il fixé, et qu'est-ce qui garantit « identique » ou « différent » ?
5. **Le fondement de l'exemption (AC 9)** et ses trois nombres codés en dur.
6. **Ce que le développeur ne saura pas.** Quelle question se posera-t-il en implémentant, à
   laquelle la spec ne répond pas ? Quel ordre des tâches le bloquerait (P8 : l'en-tête avant le
   premier `migrate run`, le squash avant les tests) ?
7. **Les manuels** — `docs/manual/fr/*.tex` et le **PDF aplati**
   (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).

## Ce que tu rends

- **Les findings**, chacun avec : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit
  exact, **la commande ou l'extrait qui l'établit**, et ce qu'il faut changer. Pour un scénario,
  montre que **son état de départ est atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Déclarer un axe non
  exercé et en tirer un finding est contradictoire. Ne qualifie jamais de « robuste » un point que
  tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans
une base persistante** — nommément `scripts/prepare-release.sh` (il bumpe les versions Cargo),
`scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`, `scripts/test-fast.sh`, tout
`git commit`/`push`/`checkout`/`stash`, `sqlx migrate`, `cargo test`/`cargo nextest`. Seule
exception : la base jetable `_p2_scratch` de l'axe 3, créée et **supprimée** par toi.
