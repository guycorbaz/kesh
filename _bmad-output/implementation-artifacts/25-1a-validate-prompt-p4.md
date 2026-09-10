# Prompt de la passe 4 de validation — Story 25-1a

- **Modèle** : Haiku 4.5, contexte frais (P1 : Sonnet + Haiku · P2 : Opus 5 · P3 : Sonnet 4.6)
- **Cible** : le seul commit de remédiation `8092cdf7` — **passe CIBLÉE**
- **Trend** : `2 CRIT / 3 HIGH` → `0 / 3 HIGH` → `0 / 1 HIGH` → ?

---

Tu es une lentille de **validation de spécification**, en contexte frais, sur le dépôt Kesh
(`/home/gcorbaz/devel/kesh`). Tu réponds en français.

CIBLE : le commit **`8092cdf7`**. Lis-le : `git show 8092cdf7`. La spec est
`_bmad-output/implementation-artifacts/25-1a-piste-inalterable.md`.

⛔ **RÈGLE ABSOLUE — GREP GROUND-TRUTH.** Avant d'affirmer qu'un texte **manque**, qu'un site
**existe** ou qu'un décompte est **faux**, vérifie-le avec `grep -nF` (fixed-string obligatoire) et
**cite la commande ET sa sortie**. Un finding non vérifié au sol sera écarté sans discussion. Le
`CLAUDE.md` explique pourquoi cette consigne t'est adressée (§ « Haiku-specific guardrails »).

⚠️ **Kesh n'est PAS en production.** Un finding dont le dommage suppose des données réelles est
réel mais **différé** (au plus MEDIUM, en le disant). Un finding portant sur une **affirmation
fausse** garde toute sa sévérité.

## ⛔ Ton axe principal : l'inventaire est-il ENFIN complet ?

Cette story tient un inventaire des documents publiés qui affirment que le journal d'audit est
inaltérable. **Il a été faux TROIS FOIS de suite, et pour trois raisons différentes** :

| passe | ce qui était annoncé | pourquoi c'était faux |
|---|---|---|
| 1 | « quatre documents » | inventaire incomplet |
| 2 | « sept mentions, cinq à traiter » | total non dérivable de sa propre énumération |
| 3 | mot-clé `inaltérable\|immutable\|insert-only` | **ne pouvait pas attraper « infalsifiable » ni « immuable »** |

La passe 3 a élargi le grep aux synonymes et rend **dix mentions dans six fichiers**.

**Ta question : y a-t-il un TROISIÈME cercle ?** Le concept peut s'exprimer sans aucun de ces
mots — par exemple *« aucune entrée ne peut être supprimée »*, *« ne se modifie plus »*,
*« conservation 10 ans »*, *« intangible »*, *« non modifiable »*, *« en lecture seule »*,
*« append-only »*, *« insert only »*, *« garantit l'intégrité »*, ou une périphrase. ⇒ **Cherche le
CONCEPT, pas un mot** : balaie `docs/manual/fr/*.tex`, `website/*.html`, `README.md`, le PRD, les
`.ftl` des **quatre** locales, les doc-comments Rust, et les textes d'écran Svelte.

⚠️ **Un faux positif est un coût, pas une faute** — la passe 3 en a écarté un (`about.html` parle
des « immutable change logs » du **processus BMAD**). Signale ce que tu trouves **et** ce que tu
écartes, avec le motif.

## Tes autres axes

1. **La ventilation se recoupe-t-elle ?** La spec annonce « 10 mentions, 6 fichiers — 6 à traiter,
   2 différées à la 25-1b, 2 sans action ». **Recompte** : 6 + 2 + 2 = 10, et le tableau du volet B
   porte 7 lignes de manuel. Les deux décomptes sont-ils cohérents entre eux ?
2. **Les deux glossaires ajoutés** (`user-manual.tex:1726`, `admin-manual.tex:2179`) : les
   citations sont-elles exactes, et leur colonne « après la story » est-elle juste ?
3. **La nouvelle caractérisation de `README.md:218`** — « affirmation positive et doublement
   fausse ». Exacte ? Le second membre (« consultable ») dépend-il vraiment de la 25-1c ?
4. **Les titres d'AC et de tâche** ont été réalignés (« SEPT sites de manuel », « les SIX sites à
   traiter »). Reste-t-il un décompte incohérent ailleurs dans la spec, l'epic ou le suivi ?

## Règles de méthode

- Tu peux exécuter `grep`, `cat`, `sed`, `find`, `pdftotext`, `gh issue view`.
- ⛔ **N'écris aucun fichier, ne commite rien, ne modifie aucune issue.**
- ⛔ Ne lance ni la suite complète ni Playwright. ⛔ N'exécute **jamais** `scripts/prepare-release.sh`.
- Lis `/home/gcorbaz/devel/kesh/CLAUDE.md`.

## Format

Sans préambule. Par finding : identifiant (P4-n), **sévérité**, **site**, **démonstration**
(commande + sortie), **conséquence**, **remède**. Puis (a) tableau des sévérités, (b) **l'inventaire
complet que tu as trouvé**, avec les faux positifs écartés et leur motif, (c) ce que tu as vérifié
et trouvé exact, (d) tes limites — en disant **quels axes tu as réellement exercés**.

⚠️ **Si tu ne trouves rien au-dessus de LOW, dis-le nettement** — c'est le résultat attendu d'une
boucle qui converge, et ce serait la première fois sur cette story. **Ne fabrique pas de sévérité
pour justifier la passe**, et n'affirme aucun manque sans l'avoir grepé.
