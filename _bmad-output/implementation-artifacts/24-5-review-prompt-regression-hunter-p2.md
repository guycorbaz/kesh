# Prompt de la passe 2 de revue de spec — Story 24-5

**Versionné** conformément au § « La passe ciblée » du `CLAUDE.md` : une passe qui ne suit pas le
protocole à trois lentilles sur le périmètre complet doit laisser de quoi la rejouer et la
contester. *Une passe non vérifiable ne vaut pas mieux qu'une passe non faite.*

- **Modèle** : Opus 5, contexte frais (passe 1 : Sonnet 4.6 + Haiku 4.5)
- **Cible** : le seul commit de remédiation `99dce529`
- **Motif** : sur les trois dernières stories, la quasi-totalité des findings des passes tardives
  portait sur ce que la remédiation précédente venait d'écrire — seize sur vingt-quatre pour la
  24-4c, sept sur huit passes cumulées de l'Epic 23.

---

Tu es une lentille de revue de spécification, en **contexte frais**, sur le dépôt Kesh
(`/home/gcorbaz/devel/kesh`, comptabilité suisse, Rust + Svelte). Tu réponds en français.

CIBLE PRINCIPALE : le commit **`99dce529`** — la remédiation de la passe 1 de la story 24-5
(`_bmad-output/implementation-artifacts/24-5-comptes-de-cloture.md`). Lis-le avec
`git show 99dce529`.

⛔ **TON HYPOTHÈSE DE TRAVAIL, ET ELLE EST MESURÉE** : *le défaut que tu cherches a été introduit
par la remédiation elle-même.* Sur ce dépôt, les passes tardives ne prennent presque jamais en
défaut une décision de conception d'origine — elles trouvent ce que le patch précédent vient de
casser. Braque-toi là-dessus en priorité.

## Ce que la passe 1 a produit, et que tu dois relire d'un œil hostile

1. **Une décision neuve, `D6`** : le backfill est inscrit au registre `POST_RESTORE_BACKFILLS` en
   **classe A** (rejeu inconditionnel), après démonstration que l'exemption « hors fenêtre » est
   fausse et que la classe B est indisponible. La décision assume que le rejeu peut **écraser un
   `postable` posé à la main** par l'utilisateur, sur un argument d'asymétrie des coûts.
2. **Un AC neuf (le 9)** qui impose cette inscription, et un **AC 11 élargi** à la modification.
3. **Une renumérotation des AC** (12 → 13 critères).
4. Des précisions sur les **dix littéraux `ChartEntry`**, le **verdict d'idempotence** (`yes`), le
   **décompte de modules** (trois → quatre), la **réserve au manuel d'administration**.

## Tes axes d'attaque, par ordre de rendement attendu

- **`D6` est-elle juste ?** Vérifie AU SOL : la fenêtre d'importabilité et son calcul
  (`crates/kesh-db/src/post_restore.rs`, `last_table_creating_migration`,
  `registry_entries_are_within_import_window`), le critère de classe A/B, et la forme réelle
  qu'une entrée doit prendre. La démonstration « exemption fausse / classe B indisponible » est-elle
  exacte, ou la spec a-t-elle éliminé trop vite une troisième voie ?
- **La réserve assumée est-elle bien la bonne ?** L'argument d'asymétrie (« l'écrasement est
  bruyant et réparable, le non-rejeu est muet ») tient-il quand on l'instrumente ? Cherche le
  scénario où l'écrasement N'EST PAS réparable, ou pas visible.
- **La renumérotation a-t-elle laissé un renvoi faux ?** C'est le mode d'échec le plus documenté
  du dépôt. Vérifie CHAQUE renvoi « AC n » du document, en prose comme dans les tâches, contre le
  critère réellement porté par ce numéro.
- **La remédiation se contredit-elle avec ce qu'elle n'a pas touché ?** Le patch a modifié `D6`,
  l'AC 7, l'AC 9, l'AC 11, `T1`, `T3`, `T6`, la table des fichiers et le décompte de modules.
  Cherche, ailleurs dans le document, une phrase qui dit encore le contraire.
- **Les décomptes se recomptent-ils ?** 13 AC, 3 invariants, 8 tâches, 10 littéraux `ChartEntry`,
  65 → 66 migrations, cinq compteurs d'audit, compteur `yes` 5 → 6, deux nombres P6
  (`total` et la fenêtre, frontière 34). Recompte depuis la source, jamais depuis le texte.
- **Ce que la passe 1 a explicitement laissé ouvert** : la lentille Sonnet a déclaré n'avoir pas
  vérifié exhaustivement qu'**aucun réglage ne pointe sur un compte de la classe 9** (décision
  `D4`). Tranche-le au sol — `company_invoice_settings`, `company_dunning_settings`, les règles de
  réconciliation, les comptes liés aux comptes bancaires.

## Règles de méthode, non négociables

- **Vérifie au sol avant d'affirmer.** `grep -nF` (fixed-string) pour tout motif textuel — le code
  Rust/SQL est plein de métacaractères. Cite la commande et son résultat dans le finding.
- **Une hypothèse éliminée par raisonnement n'est pas une hypothèse testée.** Si tu écartes une
  piste, dis comment tu l'as écartée.
- Lis `/home/gcorbaz/devel/kesh/CLAUDE.md`, au moins « Migration breaking policy » (P1 à P8),
  « Review Iteration Rule » et « Recompter ses propres comptes rendus ».

## Format de sortie

Sans préambule. Pour chaque finding : identifiant (P2-1, P2-2, …), **sévérité**
(CRITICAL/HIGH/MEDIUM/LOW), le **site** (fichier + ligne), la **démonstration** (ce que tu as lu ou
grepé, cité), la **conséquence**, le **remède**.

Termine par : (a) un tableau des sévérités ; (b) **la part des findings qui naissent de la
remédiation de la passe 1** plutôt que de la conception d'origine — c'est la mesure que ce dépôt
suit d'une story à l'autre ; (c) ce que tu as vérifié et trouvé exact ; (d) tes limites.
