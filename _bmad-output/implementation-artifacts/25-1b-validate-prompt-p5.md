# Prompt — passe 5 CIBLÉE de `bmad-create-story validate`, Story 25-1b

*Versionné le 2026-09-12. Lentille unique (Opus 5), contexte frais. **Passe ciblée** au sens de la
§ « La passe ciblée » du `CLAUDE.md` : une seule lentille, braquée sur la dernière remédiation, en
remplacement du protocole complet — la boucle converge (`1C/1H/3M/2L → 0C/3H/5M/6L → 0C/1H/2M/2L
→ 1M`).*

Tu es un **valideur adversarial** en contexte frais. Dépôt `/home/gcorbaz/devel/kesh`, spec
`_bmad-output/implementation-artifacts/25-1b-trous-alimentation.md`.

## ⛔ Ton périmètre est la REMÉDIATION, pas la story

Lis **`git show 95b87e0c`** et **`git show 79f01dfc`** — les patches des passes 3 et 4 — et
rien d'autre en premier lieu. *Ce qui reste à relire n'est plus la spécification : c'est ce qu'on
vient d'écrire dessus.* Le motif est mesuré dans ce dépôt et il est écrasant.

Les cinq objets à interroger :

1. **T6 et l'interdiction de poser la trace dans `update_step`.** Le patch dit « l'audit se pose
   dans le handler, jamais dans `update_step` ». Est-ce **réalisable** ? Le handler a-t-il tout ce
   qu'il faut ? L'enveloppe mince proposée laisse-t-elle les neuf autres appelants intacts, y
   compris `kesh-seed` ?
2. **Les SEPT sites de manuel de l'AC 12.** Vérifie les sept **au sol**, numéro de ligne et
   citation, dans le `.tex` **et** le PDF aplati. ⚠️ Puis cherche un **huitième en posant une
   QUESTION QUE LES SEPT NE POSENT PAS** : ils interrogent la couverture, les champs,
   l'attribution, l'exportabilité. Que reste-t-il ? La **conservation** (10 ans) ? La
   **consultation** ? La **restauration** ? L'**horodatage** ? *C'est ainsi que les sept ont été
   trouvés, un par un.*
3. **La portée déclarée de « 1 partielle »** et l'entrée des trois routes de `test_endpoints.rs`
   hors du diff automatique. Le patch dit-il vrai, et suffit-il ?
4. **Le périmètre à 14 routes** : la propagation de 13 → 14 est-elle **complète** ? Un énoncé
   dépendant du total a-t-il survécu ? ⚠️ C'est le défaut que la story sœur a payé **quatre fois**.
5. **Les décomptes** : 14 routes, 12 AC, 11 tâches, 7 sites de manuel, 94 appels d'audit dont 8
   conditionnels. Recompte **chacun** depuis la source. Un total doit être cohérent avec sa propre
   ventilation.

## Puis, et seulement ensuite

Si le budget le permet : ce que les quatre passes n'ont pas vu. Mais **ne dilue pas** — une passe
ciblée qui redevient générale perd ce qui la justifie.

## Ce que tu rends

- **Les findings** : sévérité, endroit exact, **la commande ou l'extrait qui l'établit**, le
  correctif. Pour chacun : **né d'un patch des passes 3-4**, ou d'origine ?
- **Ton verdict sur la clôture** : la boucle peut-elle se clore, et **sur quel fondement** ? Le
  critère est *plus aucun finding au-dessus de LOW* — dis-le franchement, dans un sens ou dans
  l'autre.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne clôt rien. **Ne qualifie jamais de « vérifié » un point que tu n'as pas
  exécuté.**

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh`, `scripts/install-hooks.sh`, `scripts/test-fast.sh`, tout
`git commit`/`push`/`checkout`/`add`/`stash`, `sqlx migrate`, toute écriture en base, et tout
script de `scripts/`. `cargo check` et `cargo clippy` sont autorisés — rien d'autre.
