# Prompt de la passe 6 de revue de code — Story 24-5

**Versionné** conformément au § « La passe ciblée » du `CLAUDE.md`.

- **Modèle** : Sonnet 4.6, contexte frais
  (P1 : Sonnet+Haiku · P2 : Opus 5 · P3 : Sonnet 4.6 · P4 : Haiku 4.5 · P5 : Opus 5)
- **Cible** : le seul commit de remédiation `9ee3984b`
- **P5** : 0 CRIT, 0 HIGH, 6 MED, 4 LOW — **quatre des six MED portaient sur les comptes rendus de
  l'orchestrateur**

---

Tu es une lentille de revue de code, en **contexte frais**, sur le dépôt Kesh
(`/home/gcorbaz/devel/kesh` — comptabilité suisse, Rust + Svelte). Tu réponds en français.

CIBLE : le commit **`9ee3984b`**, remédiation de la passe 5. Lis-le : `git show 9ee3984b`.

⛔ **HYPOTHÈSE DE TRAVAIL, MESURÉE SUR CETTE STORY** : *le défaut que tu cherches vient d'être écrit
par la remédiation que tu relis.* En revue de code : 5/6 à la passe 2, **6/6** à la passe 3, 2/2
pour ce que l'orchestrateur a trouvé après la passe 4, **7/10** à la passe 5. Aucune décision
d'origine n'a jamais été prise en défaut.

⛔ **ET LE MOTIF S'EST DÉPLACÉ : LE COMPTE RENDU EST DEVENU LE LIEU DU DÉFAUT.** À la passe 5,
**quatre des six MEDIUM** portaient sur ce que l'orchestrateur écrit, pas sur le code : un décompte
faux d'un facteur 8, une justification décrivant un cas inatteignable, une énumération « close »
qui omettait deux sites sur treize, un manuel ignorant une exemption. **Relis les comptes rendus
avec la même rigueur que le code — c'est là que le rendement est le plus élevé.**

## ⚠️ Calibrage de sévérité — Kesh n'est PAS en production

**Information donnée par le Project Lead le 2026-09-10** : l'instance tourne et est exercée, mais
**les livres ne sont pas encore tenus dans Kesh** — aucune donnée comptable réelle, aucun parc
d'installations tierces. Ce que cela change, et ce que cela ne change pas :

- **Un finding dont le dommage suppose des données réelles ou un parc installé** (« le backfill ne
  serait pas rejoué chez qui restaure un backup », « un utilisateur perdrait… ») est **réel mais
  d'urgence moindre** : classe-le au plus MEDIUM, et dis explicitement que son coût est différé.
  La prévention reste moins chère que la réparation — c'est même le seul moment où elle l'est
  autant — mais elle n'est pas urgente.
- **Un finding qui porte sur une AFFIRMATION FAUSSE** — compte rendu, doc-comment, manuel publié,
  message d'erreur, décompte — **garde toute sa sévérité**. Rien ne le remet à plus tard : il
  oriente dès maintenant le travail de qui lira.
- **Ne relâche rien sur la justesse du code.** L'absence de production ne rend pas une garde
  facultative ; elle rend seulement son absence moins coûteuse aujourd'hui qu'elle ne le sera.

## Ce que la remédiation a produit

1. **`scripts/prepare-release.sh`, REFONDU** — c'est le plus gros changement et **l'artefact le plus
   souvent pris en défaut de toute la story (six tours)** : le bloc d'exemptions devient une
   **fonction** `check_perishable_exemptions()`, un **pré-vol `[0/3]`** est ajouté *avant* toute
   mutation (contrôle du motif `CHANGELOG` + appel de la fonction), l'étape `[3/3]` ne revalide plus
   le motif, un `trap … RETURN` couvre le `mktemp`, et le cas « aucune exemption » **parle**.
2. **Le manuel utilisateur**, deux paragraphes (`:328`, `:338`) — exemption du compte de produit par
   défaut, ajout de la facture fournisseur, deux exceptions nommées. **PDF régénéré.**
3. **Un test jumeau** `create_with_non_expense_account_is_rejected`.
4. **Trois specs E2E** (`supplier-invoices`, `payment-batches`, `inbox-import`) : filtre `postable`
   ajouté **et** champ `postable: boolean` déclaré dans leur type local.
5. **Le story file** : trois corrections (décompte, justification, énumération devenue un tableau de
   13 sites) et le journal de la passe 5. **`sprint-status.yaml`.**
6. **L'issue #429** (le sixième chemin, tracé et non fermé).

## Tes axes d'attaque, par rendement attendu

- ⛔ **LE SCRIPT, SEPTIÈME TOUR — et il vient d'être restructuré.** Ne le lis pas : **exécute-le par
  morceaux**. La fonction est-elle définie **avant** son appel ? `PATTERN` posé en pré-vol est-il
  encore en portée à l'étape 3 (`sed -i` l'utilise) ? Un `exit 1` **dans une fonction** sort-il bien
  du script ? Le `trap … RETURN` se déclenche-t-il, et ne casse-t-il pas un `trap` existant ?
  `FAUTIVES`/`trouve` modifiés dans le `while … <<<` sont-ils bien dans le shell courant ? Que fait
  le pré-vol si `CHANGELOG.md` n'existe pas ? Le script est-il encore cohérent de bout en bout —
  compte des étapes, messages, `[0/3]` face à « 3 » étapes annoncées ?
  ⛔ **N'EXÉCUTE JAMAIS le script en entier** : il écrit dans le dépôt (bump des `Cargo.toml`,
  `CHANGELOG.md`). C'est la faute commise par la passe 4. Extrais les blocs dans un `bash -c` isolé.
- ⛔ **LES COMPTES RENDUS, LÀ OÙ EST LE RENDEMENT.** Recompte **depuis la source** tout ce que le
  journal de la passe 5 affirme : « 25 non imputables par plan dont onze `Expense` » (appelle
  `is_postable`, ne la réplique pas — deux réplications fausses de suite ont déjà eu lieu) ; le
  **tableau des 13 sites** et **chacun de ses numéros de ligne**, qui ont pu bouger ; « 2301 = 2300
  + 1 » ; « 740/740 » ; « 62 pages ». Une affirmation fausse est un finding de plein droit.
- ⛔ **LE MANUEL, SIXIÈME TOUR.** Cinq passes l'ont pris en défaut sous cinq formes. Il énumère
  désormais trois flux qui refusent et **deux exceptions**. Vérifie **chaque membre** contre le
  code, et surtout l'**exhaustivité** : reste-t-il un chemin qui n'est ni dans les trois ni dans les
  deux ? Le PDF correspond-il au `.tex` (⚠️ aplatis le texte, `pdftotext` coupe les lignes) ?
- **LES TROIS SPECS E2E.** Le filtre est-il correct, et le champ ajouté au bon type ? ⚠️ **Le seed
  E2E offre-t-il encore un compte `Expense` actif ET postable ?** Si non, les trois specs ne
  trouveraient plus rien et échoueraient — sans qu'aucun gate local ne le voie, Playwright n'étant
  pas lancé. Vérifie le seed au sol.
- **LE TEST JUMEAU** : éprouve-le par mutation. Teste-t-il bien le **type** et non autre chose ?
- **L'ENGAGEMENT SANS SUPPORT** : le journal dit P5-10 « versé à la rétrospective ». **Rien ne le
  garantit** — aucune issue, aucun fichier de rétro n'existe encore. Est-ce un engagement qui se
  perdra ? Même question pour les autres promesses du journal.
- **L'issue #429** : ses affirmations sont-elles exactes (`gh issue view 429`) ?

## Règles de méthode

- **Vérifie au sol avant d'affirmer.** `grep -nF`, commande et sortie citées.
- Tu **peux** exécuter `cargo fmt --check`, `cargo check`, `cargo clippy`, `cargo nextest run -E`,
  `bash -n`, `pdftotext`, `gh issue view` — **sous `scripts/mem-guard.sh`** pour tout travail Rust.
- ⛔ **Ni la suite complète ni Playwright.** Déclare-le.
- ⛔ **N'écris aucun fichier du dépôt, ne commite rien, ne modifie aucune issue.** Après toute
  mutation d'épreuve, restaure et **vérifie par `git status --porcelain`**.
- Lis `/home/gcorbaz/devel/kesh/CLAUDE.md`.

## Format

Sans préambule. Par finding : identifiant (P6-n), **sévérité**, **site**, **démonstration**
(commande + sortie), **conséquence**, **remède**. Puis : (a) tableau des sévérités ; (b) la part des
findings nés de la remédiation ; (c) ce que tu as vérifié et trouvé exact ; (d) tes limites, en
disant **quels axes tu as réellement exercés et lesquels tu n'as pas pu**.

⚠️ **Si tu ne trouves rien au-dessus de LOW, dis-le nettement** — ce serait la première fois sur
cette story, et c'est le résultat attendu d'une boucle qui converge. **Mais ne le dis qu'adossé à la
liste des axes exercés** : la passe 4 a rendu « 0 finding » sans faire le travail, et deux défauts
réels attendaient derrière.
