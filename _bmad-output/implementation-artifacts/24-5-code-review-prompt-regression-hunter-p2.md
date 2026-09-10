# Prompt de la passe 2 de revue de code — Story 24-5

**Versionné** conformément au § « La passe ciblée » du `CLAUDE.md` : une passe qui ne rejoue pas le
protocole à trois lentilles sur le périmètre complet doit laisser de quoi la rejouer et la
contester.

- **Modèle** : Opus 5, contexte frais (P1 : Sonnet 4.6 ×2 + Haiku 4.5)
- **Cible** : le seul commit de remédiation `2bf9568d`
- **P1** : 0 CRIT, 1 HIGH, 2 MED, 2 LOW

---

Tu es une lentille de revue de code, en **contexte frais**, sur le dépôt Kesh
(`/home/gcorbaz/devel/kesh` — comptabilité suisse, Rust + Svelte). Tu réponds en français.

CIBLE : le commit **`2bf9568d`** — la remédiation de la passe 1 de revue de code de la Story 24-5
(#375). Lis-le avec `git show 2bf9568d`. Le code revu en passe 1 est `2ba9613d`.

⛔ **HYPOTHÈSE DE TRAVAIL, MESURÉE SUR CE DÉPÔT** : *le défaut que tu cherches vient d'être écrit
par la remédiation que tu relis.* En revue de spec de cette même story, onze findings sur treize
(P2) puis trois sur quatre (P3) étaient de cette nature, et **aucune** décision antérieure n'a
jamais été prise en défaut. Braque-toi là-dessus.

## Ce que la remédiation a produit

1. **L'invariant I1 amendé** — il ne vaut plus que pour la **saisie manuelle**, les flux de
   réconciliation ne vérifiant pas `postable`.
2. **Le manuel utilisateur corrigé** (`docs/manual/fr/user-manual.tex`) : l'affirmation absolue
   « n'acceptent aucune écriture » devient « refuse toute écriture **saisie à la main** », plus un
   `keshnote` qui énonce la réserve sur les flux automatiques. **PDF régénéré.**
3. **L'issue #427** ouverte pour la fermeture des flux automatiques (catégorie A, hors périmètre).
4. **Un rappel dans `scripts/prepare-release.sh`** : il compte les exemptions de
   `EXEMPT_MIGRATIONS` dont la justification porte le marqueur textuel **« SE PÉRIME »** et les
   affiche avant le tag.
5. **Une assertion sur le code d'erreur** dans `posting_to_a_closed_closing_account_is_refused`
   (`INACTIVE_OR_INVALID_ACCOUNTS`).
6. **Deux résidus P6 de plus** corrigés dans `crates/kesh-db/tests/migrations_upgrade_path.rs`.

## Tes axes d'attaque, par rendement attendu

- **Le manuel dit-il VRAI maintenant ?** C'est là qu'était le HIGH de la passe 1, et la
  remédiation y a écrit du texte neuf. Lis le passage **entier** et son voisinage
  (§ « Rôles des comptes », § « Reprise de comptabilité », § « Les comptes de clôture »). Cherche
  une nouvelle contradiction, une affirmation trop large, ou un renvoi devenu faux. ⚠️ **Le PDF
  a-t-il réellement été régénéré depuis le `.tex` corrigé ?** Vérifie-le.
- **Le rappel de `prepare-release.sh` fonctionne-t-il ?** Exécute-le mentalement, et vérifie au sol
  que le `grep -c "SE PÉRIME"` trouve bien ce qu'il prétend. Que se passe-t-il si la justification
  est reformulée, ou si l'accent de « PÉRIME » change ? Un garde-fou accroché à une chaîne
  française accentuée est-il robuste ? Le script est-il toujours syntaxiquement valide (`bash -n`) ?
- **L'assertion neuve est-elle juste ?** `INACTIVE_OR_INVALID_ACCOUNTS` est-il réellement le code
  rendu par ce chemin ? Vérifie le mappage `DbError` → `AppError` → code JSON, au sol. ⚠️ Une
  assertion **fausse** ferait rougir le gate ; une assertion sur le **mauvais** code le ferait
  passer en mesurant autre chose.
- **L'amendement de I1 va-t-il assez loin — et pas trop loin ?** Dit-il vrai sur le périmètre
  réellement couvert ? Les AC 12 et 13, et le reste de la spec, sont-ils cohérents avec lui, ou
  reste-t-il ailleurs une phrase qui promet ce que le code ne tient pas ?
- **Reste-t-il des résidus P6 ?** La passe 1 en a trouvé deux après quatre déclarés. Grep le motif
  **structurel** (`total - <N>`, `total == <N>`, « les N dernières »), pas les valeurs. ⚠️ Deux
  occurrences (`total == 39`, `total - 8`) sont des **généalogies historiques délibérées** — les
  signaler serait un faux positif.
- **L'issue #427 dit-elle vrai ?** Lis-la (`gh issue view 427`) et vérifie ses affirmations au sol.
  Une issue fausse oriente mal le travail futur.
- **Les décomptes** : 13 AC, 3 invariants, 8 tâches, 14 tests neufs, 2295 tests, 66 migrations,
  cinq compteurs d'audit. Recompte depuis la source.

## Règles de méthode

- **Vérifie au sol avant d'affirmer.** `grep -nF` pour tout motif textuel ; cite commande et
  résultat. Tu **peux** exécuter `cargo fmt --check`, `cargo check`, `cargo clippy` et `bash -n`.
  Tu **ne peux pas** lancer la suite complète ni Playwright : dis-le plutôt que de supposer.
- **Une hypothèse éliminée par raisonnement n'est pas une hypothèse testée.**
- Lis `/home/gcorbaz/devel/kesh/CLAUDE.md` — « Migration breaking policy », « Review Iteration
  Rule », « Recompter ses propres comptes rendus », « Propagation post-patch ».

## Format

Sans préambule. Par finding : identifiant (P2-n), **sévérité**, **site**, **démonstration**
(commande + résultat), **conséquence**, **remède**. Puis : (a) tableau des sévérités ; (b) **la part
des findings nés de la remédiation** ; (c) ce que tu as vérifié et trouvé exact ; (d) tes limites.

⚠️ **Si tu ne trouves rien au-dessus de LOW, dis-le nettement** — c'est le résultat attendu d'une
boucle qui converge. Ne fabrique pas de la sévérité pour justifier la passe.
