# Prompt — passe 2 de `bmad-create-story validate`, Story 25-1c-a

*Versionné le 2026-09-15. Une lentille en contexte frais (Opus), orthogonale aux deux lentilles de
la passe 1 (Sonnet, Haiku).*

Tu es un **valideur adversarial** en contexte frais. Ton objet est le fichier
`_bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-1c-a-journal-audit-route`. Ta mission n'est pas
d'approuver : c'est de **trouver ce qui ferait échouer, dévier ou mentir** l'implémentation qui suivra.

⛔ **Rien ne se croit sur parole** — ni la spec, ni son Change Log, ni les vérifications de la passe 1
qu'il rapporte. Chaque affirmation se **vérifie dans le code**.

⚠️ **Ne conteste PAS les arbitrages du Project Lead** (`epic-25-vague1-suite.md` § *Arbitrages du
2026-09-15*). Les deux choix **non arbitrés** — inclusion des entrées sans société, audit de l'export —
peuvent être contestés sur pièces.

## Où regarder d'abord

*La sévérité se déplace vers ce qu'on vient d'écrire.* La passe 1 a trouvé un **CRITICAL** — la sonde
des parenthèses de l'AC 19 (c) ne sondait rien — et a réécrit : l'AC 19 (c), la ligne correspondante
du tableau de l'AC 21, l'AC 17 (deux sites du manuel, dont un titre de sous-section), l'AC 14 (maintien
de `for_actor`), une référence de ligne de l'AC 1. **Ta base de comparaison est le commit de la
spécification** : `git diff 25a61a53 -- _bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md`.
Nomme-la dans ton rapport.

## Les axes, et tu déclareras lesquels tu as exercés

1. **La remédiation de la passe 1.** La nouvelle sonde des parenthèses tranche-t-elle **vraiment** ?
   Rejoue-la sur une base jetable : avec parenthèses, sans parenthèses. Le reste du tableau de
   mutations de l'AC 21 est-il juste — **chaque** ligne, et en particulier celle de la borne de date :
   la mutation `< date_to + 1 j` → `<= date_to` fait-elle rougir l'AC 19 (d) tel qu'il est écrit ?
2. **Les autres assertions de l'AC 19 et de l'AC 20**, une par une : qu'est-ce qui la rendrait fausse ?
   Est-elle vraie par construction du montage ? Les identifiants sont-ils désalignés là où il le faut ?
3. **Le RBAC en profondeur** : `HEAD` et `OPTIONS` sur les deux routes, une clé `read-write`, un jeton
   expiré ; la route d'export est-elle joignable par un chemin que la spec n'a pas pensé ?
4. **Le DTO et le CSV** : `createdAt` en UTC explicite (le patron `repositories/invoices.rs:89` existe) ;
   `details_json` en JSON compact dans une cellule passée à `csv_sanitize` — un JSON commençant par
   `{` ou `[` est-il affecté ? un `details` `null` ? une cellule numérique (`entityId` négatif ?) ?
5. **Les manuels** : l'AC 17 nomme maintenant deux sites. Y en a-t-il un troisième — la procédure
   d'import elle-même, le glossaire, la brochure, le README, `website/` en anglais ? Contrôle le **PDF
   aplati** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) ; en LaTeX le souligné s'écrit `\_`.
6. **Ce que le développeur ne saura pas** : quelle question se posera-t-il en implémentant, à laquelle
   la spec ne répond pas ? L'ordre des tâches le bloquerait-il (rebase sur `main` avant ou après) ?
7. **Cohérence interne et décomptes** — recompte, ne relis pas.

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif. Pour un scénario, montre que **son état de
  départ est atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Déclarer un axe non exercé
  et en tirer un finding est contradictoire. Ne qualifie jamais de « vérifié » ce que tu n'as pas
  exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une
base persistante** — nommément `scripts/prepare-release.sh` (il bumpe les versions Cargo),
`scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/`checkout`/`add`/`stash`/
`reset`/`rebase`, `sqlx migrate` sur `kesh` ou `kesh_e2e`, `cargo test`/`cargo nextest`. Seule
exception : une base jetable `_v2_scratch` que tu crées et **supprimes** toi-même.
