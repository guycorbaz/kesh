# Prompt de la passe 3 de validation — Story 25-1a

- **Modèle** : Sonnet 4.6, contexte frais (P1 : Sonnet + Haiku · P2 : Opus 5)
- **Cible** : le seul commit de remédiation `7b0e3f70` — **passe CIBLÉE**
- **P2** : 0 CRIT, 3 HIGH, 3 MED, 3 LOW — dont **7 sur 9 nés de la remédiation de la P1**

---

Tu es une lentille de **validation de spécification**, en contexte frais, sur le dépôt Kesh
(`/home/gcorbaz/devel/kesh`). Tu réponds en français.

CIBLE : le commit **`7b0e3f70`** — la remédiation de la passe 2. Lis-le : `git show 7b0e3f70`.
La spec est `_bmad-output/implementation-artifacts/25-1a-piste-inalterable.md`.

⛔ **HYPOTHÈSE DE TRAVAIL, MESURÉE** : *le défaut que tu cherches vient d'être écrit par la
remédiation que tu relis.* Passe 1 → passe 2 : **7 findings sur 9** étaient de cette nature, et
**aucune décision d'origine n'a jamais été prise en défaut**. Braque-toi là-dessus.

⚠️ **Kesh n'est PAS en production.** Un finding dont le dommage suppose des données réelles est
réel mais **différé** (au plus MEDIUM, en le disant). Un finding portant sur une **affirmation
fausse** garde toute sa sévérité.

## Ce que la remédiation a produit — et qu'il faut contrôler

1. **Le volet B entièrement refondu** : un tableau de **cinq** sites publiés, avec pour chacun un
   statut « aujourd'hui » et « après la story ». ⛔ **Vérifie chaque cellule des deux colonnes.**
   Une case « après la story » est une **prédiction** : est-elle juste ? Le site
   `admin-manual.tex:1803-1808` deviendra-t-il vraiment faux ? `user-manual:1597` deviendra-t-il
   vraiment vrai *sur une instance finalisée* ?
2. **L'AC 2 réécrite** : elle pose désormais un **résultat** (« après l'import d'un backup
   étranger, chaque entrée antérieure existe encore et nomme son auteur d'origine ») au lieu d'un
   mécanisme. **Est-elle décidable ?** Peut-on la déclarer tenue ou non tenue sans ambiguïté ?
   Est-elle **réalisable** — ou vient-on de poser une exigence qu'aucun mécanisme ne satisfait ?
3. **L'AC 8, et son fait central** : le handler refuserait tout reset à `step_completed >= 7`
   (`onboarding.rs:257`). ⛔ **Vérifie-le, et vérifie surtout sa PORTÉE** : une instance réellement
   en service a-t-elle toujours `step_completed >= 7` ? Que vaut ce champ après un `finalize` ?
   Existe-t-il un chemin qui le fasse **redescendre** ? Si oui, la nuance du manuel est fausse.
4. **L'AC 9** : deux sites sont renvoyés à la 25-1b parce qu'ils parlent d'**intégrité**. Le
   découpage est-il juste — ou l'un des trois autres en dépend-il aussi ?
5. **Les corrections de référence** (six annoncées, cinq appliquées) et **le décompte** « sept
   mentions, cinq à traiter ». **Recompte depuis la source.**
6. **Les tâches T4 à T8** réécrites, dont les garde-fous de migration conditionnels.

## Règles de méthode

- **Vérifie au sol avant d'affirmer** : `grep -nF`, commande **et** sortie citées.
- Tu peux exécuter `grep`, `cat`, `sed`, `pdftotext`, `gh issue view`, et `cargo check`/`clippy`
  **sous `scripts/mem-guard.sh`**.
- ⛔ **N'écris aucun fichier, ne commite rien, ne modifie aucune issue.**
- ⛔ Ne lance ni la suite complète ni Playwright. ⛔ N'exécute jamais `scripts/prepare-release.sh`.
- Lis `/home/gcorbaz/devel/kesh/CLAUDE.md`.

## Format

Sans préambule. Par finding : identifiant (P3-n), **sévérité**, **site**, **démonstration**
(commande + sortie), **conséquence**, **remède**. Puis (a) tableau des sévérités, (b) la part des
findings nés de la remédiation, (c) ce que tu as vérifié et trouvé exact, (d) tes limites — en
disant **quels axes tu as réellement exercés**.

⚠️ **Si tu ne trouves rien au-dessus de LOW, dis-le nettement** — c'est le résultat attendu d'une
boucle qui converge, et ce serait la première fois sur cette story. **Mais ne le dis qu'adossé à la
liste des axes exercés.**
