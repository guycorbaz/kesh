# Prompt de la passe 4 de revue de spec — Story 24-5

**Versionné** conformément au § « La passe ciblée » du `CLAUDE.md`.

- **Modèle** : Haiku 4.5, contexte frais (P1 : Sonnet + Haiku ; P2 : Opus 5 ; P3 : Sonnet 4.6)
- **Cible** : les deux commits de remédiation de la passe 3 — `85b83904` et `4ccddb19`
- **Trend** : 6 → 13 → 4. Sévérité maximale : CRITICAL → HIGH → HIGH.

---

Tu es une lentille de revue de spécification, en **contexte frais**, sur le dépôt Kesh
(`/home/gcorbaz/devel/kesh`, comptabilité suisse, Rust + Svelte). Tu réponds en français.

CIBLE : `git show 85b83904` puis `git show 4ccddb19` — la remédiation de la passe 3 de la story
24-5 (`_bmad-output/implementation-artifacts/24-5-comptes-de-cloture.md`).

⛔ **HYPOTHÈSE DE TRAVAIL, MESURÉE** : *le défaut que tu cherches vient d'être écrit par la
remédiation que tu relis.* En passe 2, onze findings sur treize étaient de cette nature ; en
passe 3, trois sur quatre. **Aucune** décision antérieure n'a jamais été prise en défaut.

## ⛔ RÈGLE ABSOLUE — VÉRIFICATION AU SOL AVANT TOUT FINDING CRITICAL OU HIGH

Le dépôt a mesuré, sur ce modèle, **quatre hallucinations CRITICAL/HIGH** réfutées par grep.
Si tu affirmes qu'un code est absent, qu'une garde manque, qu'un fichier ne contient pas
quelque chose, ou qu'un comportement existe, tu **DOIS** d'abord l'établir par
`grep -nF "<chaîne exacte>" <fichier>` — le flag `-F` est **obligatoire**, le code Rust/SQL/LaTeX
étant plein de métacaractères (`.`, `*`, `(`, `[`, `\`, `{`) — ou par lecture directe. **Cite la
commande et son résultat dans le finding.** Un CRITICAL/HIGH sans vérification au sol sera rejeté
d'office.

## Ce que la remédiation de la passe 3 a produit

1. **D6 REFONDUE** : la décision passe de *classe A avec dérogation P7* à **exemption
   (`EXEMPT_MIGRATIONS`) justifiée sur le PARC** — au motif que (a) `admin-manual.tex:1609`
   **promet** que les données ne sont jamais écrasées, et (b) la dernière version publiée
   (v0.11.1, 2026-08-24) est **antérieure** à la borne `20260827000001`, donc l'intervalle ne
   contient aucun binaire distribué.
2. **L'AC 9 réécrite** (exemption au lieu d'inscription au registre), **T3 refait** (trois puces :
   exemption, vérification du fait par `git tag`, interdiction d'inscrire au registre).
3. **T6 allégé** : plus rien à corriger dans le manuel d'administration ; le chemin d'écran passe
   de « Réglages → Soldes de départ » à **« Administration → Soldes de départ »**.
4. **La table des fichiers** : `admin-manual.tex` retiré, la ligne `post_restore.rs` réécrite.
5. **Le journal de la passe 1 daté** d'une note disant que sa décision a été renversée.
6. `4ccddb19` : la question du parc **tranchée** (le NAS exécute v0.11.1).

## Axes d'attaque

- **L'exemption est-elle bien formée ?** Lis `EXEMPT_MIGRATIONS` et ses deux tests
  (`every_data_backfill_migration_is_triaged`, `exemptions_claiming_out_of_window_really_are_out_of_window`)
  dans `crates/kesh-db/src/post_restore.rs`. La justification prescrite passerait-elle réellement
  les deux ? Le format attendu (tuple `(i64, &str)`) est-il respecté par ce que la spec demande ?
- **Le raisonnement du parc tient-il ?** Vérifie les dates au sol (`git tag`, `git log`). Y a-t-il
  un cas où une archive porterait `postable = TRUE` **sans** venir d'un binaire de l'intervalle ?
  Pense aux imports partiels, aux bases de dev, aux sociétés créées avant/après.
- **La cohérence interne après DEUX renversements.** D6 a changé d'avis deux fois. Cherche, partout
  dans le document — AC, invariants, tâches, Dev Notes, journaux de revue —, une phrase qui décrit
  encore l'ancienne décision (classe A, registre, dérogation, réserve au manuel d'administration,
  en-tête de module à amender).
- **Les renvois** : chaque « AC n », chaque `D1..D6`, `T1..T8`, `I1..I3`, `P1..P8`, chaque numéro de
  ligne de fichier. Un par un, contre sa cible réelle.
- **Les décomptes** : 13 AC, 3 invariants, 8 tâches, 10 littéraux `ChartEntry`, 65 → 66 migrations,
  `yes` 5 → 6, deux nombres P6 (`total` et la fenêtre, frontière 34). Recompte **depuis la
  source**, jamais depuis le texte.
- **Le chemin d'écran** : « Administration → Soldes de départ » est-il exact ? Vérifie le sidebar
  (`frontend/src/routes/(app)/+layout.svelte`) **et** ce qu'en dit `user-manual.tex`.

## Format

Sans préambule. Par finding : identifiant (P4-n), **sévérité**, **le cas**, **la vérification au
sol** (commande + résultat), **la conséquence**, **le remède**. Puis : (a) tableau des sévérités ;
(b) part des findings nés de la remédiation de la passe 3 ; (c) ce que tu as vérifié et trouvé
exact ; (d) tes limites.

⚠️ **Si tu ne trouves rien au-dessus de LOW, dis-le nettement.** C'est un résultat, pas un échec —
c'est même le résultat attendu d'une boucle qui converge. **Ne fabrique pas de la sévérité pour
justifier la passe.**
