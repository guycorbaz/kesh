# Passe 5 de `bmad-code-review` — lentille UNIQUE : Regression Hunter

**Story 22-2b** (#301) — prévention des doublons à la saisie d'un contact.
**Arbitrage de Guy, 2026-08-18** : passe ciblée, une seule lentille, au lieu du protocole à trois.

## Pourquoi cette passe existe, et pourquoi elle est ciblée

Quatre passes ont tourné. Le motif est **constant et documenté** dans le Change Log :
**chaque passe trouve un défaut INTRODUIT par la remédiation de la précédente.**

| Passe | Ce qu'elle a trouvé |
|---|---|
| 2 | les résidus de prose laissés par la remédiation de la passe 1 |
| 3 | un `CRITICAL` **introduit** par la remédiation de la passe 2 (correctif de pluriel inerte en production) |
| 4 | le ternaire posé par la passe 3, que **rien n'exerçait** — le muter réaffichait le `CRITICAL` de la passe 3 mot pour mot |

**La remédiation de la passe 4 n'a subi AUCUNE relecture.** C'est elle, et elle seule, l'objet de cette passe.

## Ton périmètre — strict

Le commit **`e207ecb7`** (« passe 4 de revue — prouver chaque maillon ne prouve pas la chaîne »).
Lis-le avec `git show e207ecb7`.

**Priorité au code neuf ou remanié** (ignore les fichiers `.md` et `.yaml` sauf s'ils contredisent le code) :

1. `crates/kesh-i18n/src/loader.rs` — **la fonction pure `cles_a_selecteur`**, neuve, et le test
   `the_select_detector_catches_all_three_fluent_forms`. Elle **remplace** une détection ligne-à-ligne
   qui ne voyait qu'une des trois formes de sélecteur Fluent.
2. `frontend/src/routes/(app)/contacts/contacts-i18n-realpath.test.ts` — **garde neuve**, sans mock
   d'`i18nMsg`, dictionnaire réel, lecture du DOM.
3. `frontend/src/lib/features/contacts/duplicate-i18n-keys.test.ts` — **garde neuve**, lien clé écrite ↔ catalogue, 4 locales.
4. `frontend/src/lib/features/contacts/duplicate-probe.ts` — garde neuve sur `rank` (valeur serveur) + journalisation du `catch`.
5. `frontend/src/routes/(app)/contacts/+page.svelte` et `contacts-page.test.ts`.
6. `frontend/scripts/mutants-22-2b.mjs` — entrées de mutation ajoutées **et une retirée**.
7. `crates/kesh-db/src/repositories/contacts.rs` — un doc-comment **déplacé** d'un test à un autre.

## Ce que tu cherches — et ce que tu ne cherches pas

**Tu cherches** : ce que la remédiation de la passe 4 a **cassé, affaibli ou mal fermé**.

- Une garde neuve qui **ne mord pas** : la muter laisse-t-elle le test vert ? Éprouve-le.
- Une garde neuve qui **mord trop** : faux positif, fragile au renommage, couplée à un détail volatil.
- `cles_a_selecteur` : les trois formes sont testées, mais **quelles formes valides de Fluent
  passent encore au travers, et quels faux positifs produit-elle ?** Un `->` dans du texte ordinaire ?
  Un attribut Fluent (`.attr = …`) ? Une valeur multi-ligne dont une ligne non indentée porte `=` ?
  Une ligne de commentaire indentée ? Un `#` en milieu de ligne ?
- Le doc-comment **déplacé** dans `contacts.rs` : décrit-il bien le test qu'il surplombe désormais,
  et le test qu'il a quitté n'est-il pas resté sans documentation qui le justifie ?
- L'entrée de mutation **retirée** du banc : ce qu'elle prétendait couvrir est-il couvert ailleurs,
  ou un trou a-t-il été ouvert ?
- La garde sur `rank` : que se passe-t-il aux bornes (absent, `null`, négatif, non entier, énorme) ?
  Le `catch` de la sonde transforme une erreur en « aucun doublon » — **muet**. La garde évite-t-elle vraiment ça ?

**Tu ne cherches PAS** : à re-réviser la story entière, ni les défauts déjà tracés
(KF-040 / #316, KF-005, #283, #315). Ne rouvre pas les quatre passes précédentes.

## Discipline non négociable

1. **Grep ground-truth avant tout `CRITICAL` ou `HIGH`** affirmant qu'un code est absent ou qu'un
   anti-pattern subsiste : `grep -nF "<chaîne exacte>" <fichier>`. Le `-F` est **obligatoire**
   (métacaractères regex fréquents en Rust/TS). Si le grep te réfute, **abandonne le finding** et
   dis-le. C'est la § *Haiku-specific guardrails* du `CLAUDE.md`, appliquée à tous les modèles.
2. **Une mutation qui laisse un test vert est une preuve ; une lecture ne l'est pas.** Quand tu
   affirmes qu'une garde ne mord pas, **exécute-la mutée** puis restaure.
3. **N'écris JAMAIS dans le dépôt de façon durable.** `git status --porcelain` doit être **vide**
   avant que tu commences et **vide** quand tu rends. Aucun commit, aucun `git add`, aucun push.
   Si tu mutes un fichier pour éprouver une garde, restaure-le immédiatement (`git checkout -- <fichier>`)
   et vérifie.
4. **Gates ciblés seulement**, jamais le gate complet :
   - Rust : `scripts/mem-guard.sh cargo test -p kesh-i18n` (ou `cargo nextest run -E 'binary(...)'`)
   - Front : `cd frontend && ../scripts/mem-guard.sh npx vitest run <chemin du fichier>`
   - **Toujours via `scripts/mem-guard.sh`** — la station a perdu deux sessions sur OOM aujourd'hui.
5. **Recompte tout décompte depuis la source.** Un total doit être cohérent avec sa ventilation.

## Ce que tu rends

Un rapport en **français**, structuré ainsi, et **rien d'autre** (pas de patch appliqué) :

- **Verdict d'ensemble** en trois lignes : la remédiation de la passe 4 tient-elle, oui ou non ?
- **Les findings**, du plus grave au moins grave. Pour chacun :
  - sévérité `CRITICAL` / `HIGH` / `MEDIUM` / `LOW`,
  - fichier et ligne,
  - **le scénario d'échec concret** (entrées → comportement faux), pas une inquiétude générale,
  - **la vérification que tu as faite** : le grep exact, ou la mutation exécutée et son résultat,
  - le correctif proposé, en une phrase.
- **Ce que tu as éprouvé sans rien trouver** — les gardes que tu as mutées et qui ont bien rougi.
  C'est aussi utile que les findings, et ça dit ce que la passe couvre réellement.
- **Ton propre décompte** : `N CRITICAL / N HIGH / N MEDIUM / N LOW`, cohérent avec ta liste.

Si tu ne trouves rien au-dessus de `LOW`, **dis-le franchement** : c'est le critère d'arrêt de la
§ *Review Iteration Rule*, et un rapport vide honnête vaut mieux qu'un finding fabriqué.
