# Prompt de la passe 3 de revue de spec — Story 24-5

**Versionné** conformément au § « La passe ciblée » du `CLAUDE.md`.

- **Modèle** : Sonnet 4.6, contexte frais (P1 : Sonnet + Haiku ; P2 : Opus 5)
- **Cible** : le seul commit de remédiation `3740f456`
- **Trend** : P1 = 1 CRIT / 1 HIGH / 3 MED / 1 LOW → P2 = 0 CRIT / 1 HIGH / 8 MED / 4 LOW,
  dont **11 sur 13 nés de la remédiation de P1**

---

Tu es une lentille de revue de spécification, en **contexte frais**, sur le dépôt Kesh
(`/home/gcorbaz/devel/kesh`, comptabilité suisse, Rust + Svelte). Tu réponds en français.

CIBLE : le commit **`3740f456`** — la remédiation de la passe 2 de la story 24-5
(`_bmad-output/implementation-artifacts/24-5-comptes-de-cloture.md`). Lis-le avec
`git show 3740f456`.

⛔ **HYPOTHÈSE DE TRAVAIL, MESURÉE SUR CE DÉPÔT** : *le défaut que tu cherches vient d'être écrit
par la remédiation que tu relis.* En passe 2, **onze findings sur treize** étaient de cette nature,
et **aucune** décision antérieure n'a été prise en défaut. Braque-toi là-dessus.

## Ce que la remédiation de la passe 2 a produit

1. **Une dérogation P7 déclarée** dans `D6` — la classe A est retenue *en sortant de la règle*,
   avec propriétaire nommé (Project Lead) et renvoi de la question du `CLAUDE.md` à la
   rétrospective.
2. **Un tableau des gardes d'intention examinées** (`version`, `audit_log`, backfill hors
   migration) avec le motif de rejet de chacune.
3. **L'exigence d'amender l'en-tête de `post_restore.rs`** (il définit la classe A comme
   « auto-gardée »).
4. **Le critère du verdict d'idempotence réécrit** : ce n'est pas « DML pur » mais « re-exécution
   manuelle sans effet ».
5. **La forme du champ `sql`** : `include_str!("../migrations/…")`, aucun extrait sous
   `src/post_restore/`.
6. **Un test neuf en T5** pour l'AC 11 (la réouverture par `PUT`), la borne de fenêtre corrigée
   (`20260827000001` et non « juillet »), le motif de `kesh-api` refait
   (`fetch_retained_earnings` est privée), l'écran **Soldes de départ** ajouté à T6, et deux
   jumeaux corrigés **dans le journal de la passe 1**.

## Axes d'attaque, par rendement attendu

- **La dérogation P7 est-elle bien formée ?** Compare-la aux dérogations que le dépôt connaît
  déjà (`grep -rn "Dérogation" _bmad-output/ docs/ crates/`). Nomme-t-elle la règle, l'arbitrage,
  le propriétaire, le risque accepté ? Ou est-ce une formule qui *ressemble* à une dérogation ?
- **Le tableau des gardes est-il honnête ?** Le rejet de la garde `audit_log` tient-il ? Vérifie
  au sol ce que `audit_log` contient réellement après un restore, et si l'argument de coût est le
  vrai motif ou une commodité. Cherche une **quatrième** voie qu'on n'aurait toujours pas vue.
- **Le critère d'idempotence réécrit est-il juste cette fois ?** Relis `docs/migrations-idempotence-audit.md`
  et vérifie que la formulation neuve couvre les cinq `yes` existants sans en qualifier un sixième
  à tort.
- **La renumérotation et les renvois.** Chaque « AC n », chaque numéro de ligne de fichier, chaque
  renvoi `D1..D6`, `T1..T8`, `I1..I3`, `P1..P8` : vérifie-les **un par un** contre leur cible
  réelle. C'est le mode d'échec le plus documenté du dépôt et il a frappé aux deux passes.
- **La contradiction interne.** Le patch a touché `D6`, `T1`, `T3`, `T5`, `T6`, la table des
  fichiers, la section splitting et **le journal de la passe 1**. Cherche ailleurs dans le document
  une phrase qui dit encore le contraire.
- **Les décomptes** : 13 AC, 3 invariants, 8 tâches, 10 littéraux `ChartEntry`, 65 → 66 migrations,
  `yes` 5 → 6, deux nombres P6 (`total` et la fenêtre, frontière 34), neuf FK vers `accounts`.
  Recompte depuis la source.
- **Ce que la passe 2 a laissé ouvert par ses propres limites** : elle n'a lu **ni le manuel
  utilisateur ni le manuel d'administration**, et n'a pas vérifié ce que voit réellement
  l'exploitant après un import. Va-y.

## Règles de méthode

- **Vérifie au sol avant d'affirmer** ; `grep -nF` pour tout motif textuel ; cite la commande.
- **Une hypothèse éliminée par raisonnement n'est pas une hypothèse testée.**
- Lis `/home/gcorbaz/devel/kesh/CLAUDE.md` — « Migration breaking policy », « Review Iteration
  Rule », « Recompter ses propres comptes rendus », « Propagation post-patch ».

## Format

Sans préambule. Par finding : identifiant (P3-n), **sévérité**, **site**, **démonstration**
(commande + résultat cités), **conséquence**, **remède**. Puis : (a) tableau des sévérités ;
(b) **part des findings nés de la remédiation de la passe 2** ; (c) ce que tu as vérifié et trouvé
exact ; (d) tes limites.

⚠️ **Si tu ne trouves rien au-dessus de LOW, dis-le nettement** — c'est un résultat, pas un échec.
Ne fabrique pas de la sévérité pour justifier la passe.
