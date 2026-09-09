# Prompt versionné — Story 24-4c, passe 4 de `validate` (passe CIBLÉE)

**Date** : 2026-08-30 · **Modèle** : Haiku 4.5, contexte frais · **Périmètre** : le seul commit
`0c2c5f54` (la remédiation de la passe 3) · **Lentille unique** : contrôle mécanique de
propagation.

## Où en est la boucle

| passe | modèle | CRIT | HIGH | MED | LOW | total | nés d'une remédiation |
|---|---|---|---|---|---|---|---|
| 1 | Sonnet 4.6 + Haiku 4.5 | 2 | 1 | 3 | 2 | **8** | — |
| 2 | Opus 5 *(ciblée)* | 0 | 4 | 6 | 3 | **13** | **13 / 13** |
| 3 | Sonnet 4.6 *(ciblée)* | 0 | 3 | 0 | 0 | **3** | **3 / 3** |
| 4 | Haiku 4.5 *(ciblée)* | — | — | — | — | — | — |

⛔ **Le mode d'échec de cette story est identifié et il est unique** : corriger la thèse au site
nommé et laisser ses applications ailleurs dans le même document. Passe 1→2, puis 2→3. Les trois
findings de la passe 3 en sont, et rien d'autre.

## Pourquoi ce prompt est MÉCANIQUE et non interprétatif

⚠️ **En passe 1, la lentille Haiku a rendu zéro finding en vérifiant la spec contre elle-même** —
elle a déclaré un cas « traité » en citant l'invariant que ce cas mettait précisément en défaut.
Deux CRITICAL lui ont échappé.

⇒ Ce prompt ne lui demande **aucun jugement de conception**. Il lui demande d'exécuter des
**commandes précises**, de rapporter leurs résultats, et de signaler les écarts. C'est ce que ce
modèle fait le mieux, et c'est exactement ce dont la fin d'une boucle a besoin : le mode d'échec
restant est **mécanique**, donc son contrôle doit l'être aussi.

## Le prompt, tel qu'il a été donné

> Tu es une lentille de **contrôle de propagation**, en contexte frais, sur le dépôt Kesh
> (`/home/gcorbaz/devel/kesh`). Réponds en FRANÇAIS.
>
> **Le document sous contrôle** :
> `_bmad-output/implementation-artifacts/24-4c-verrou-de-periode.md`.
> **Le commit à relire** : `0c2c5f54` (`git show 0c2c5f54`).
>
> ⛔ **Ta mission n'est PAS de juger la conception.** Elle est de vérifier, **commande par
> commande**, qu'aucune valeur corrigée par ce commit ne subsiste sous sa forme fausse ailleurs
> dans le document, et qu'aucune affirmation factuelle ne contredit le code.
>
> ⚠️ **Une vérification qui ne sort pas du document ne compte pas.** Citer une AC ou un invariant
> pour prouver qu'un cas est traité est un raisonnement circulaire — c'est ainsi que deux
> CRITICAL ont été manqués en passe 1. Chaque affirmation portant sur le **code** se vérifie dans
> le **code**.
>
> ### A. Les jetons corrigés par ce commit — exécute chaque commande et rapporte sa sortie
>
> ```sh
> F=_bmad-output/implementation-artifacts/24-4c-verrou-de-periode.md
> grep -nE 'create_in_tx[^_]' "$F"        # doit ne rester que : le wrapper nommé comme tel
> grep -n 'books.unlocked' "$F"           # aucun ne doit désigner l'import
> grep -n 'seize' "$F"                    # aucun hors du journal de revue
> grep -nE '\b(13|treize|22|vingt-deux|16)\b' "$F"   # décomptes : chacun se justifie ?
> ```
> Pour **chaque** ligne rendue, dis si elle est **légitime** (elle parle du wrapper, ou c'est une
> mention historique du journal) ou **résiduelle** (elle applique encore la forme fausse).
>
> ### B. Les affirmations factuelles — vérifie-les dans le CODE
>
> ```sh
> grep -n 'fn create_in_tx\|create_in_tx_inner(' crates/kesh-db/src/repositories/journal_entries.rs
> sed -n '263p;274p' crates/kesh-db/src/repositories/journal_entries.rs
> grep -c '^pub fn serialize_.*_csv' crates/kesh-api/src/exports/csv_tables.rs
> grep -n '"companies"' crates/kesh-db/src/backup.rs
> ls crates/kesh-db/migrations/*.sql | wc -l
> grep -n 'total, 64\|total - 30' crates/kesh-db/tests/migrations_upgrade_path.rs
> ```
> Compare chaque résultat à ce que le document affirme. **Cite la commande ET sa sortie.**
>
> ### C. La cohérence interne des trois tableaux
>
> 1. Chaque **AC** (elles sont numérotées 1 à 14) a-t-elle au moins une **tâche** qui la porte ?
> 2. Chaque fichier du tableau « **Fichiers à toucher** » est-il justifié par une AC ou une tâche ?
> 3. Chaque AC qui exige un comportement testable a-t-elle un **test nommé en T6** ?
> Rapporte les manques par numéro, sans les interpréter.
>
> ### D. Le journal de revue lui-même
>
> Ses décomptes se recomptent : le tableau de la passe 3 annonce « 3 HIGH » — le tableau qui suit
> en compte-t-il trois ? Les totaux 8 / 13 / 3 correspondent-ils aux lignes des tableaux ?
>
> ⛔ **Format** : pour chaque écart, `[SÉVÉRITÉ] identifiant — titre`, la **commande exécutée**,
> sa **sortie**, et le correctif en une phrase. CRITICAL / HIGH / MEDIUM / LOW. Termine par le
> tableau des comptes et par la réponse à **une seule question** : *reste-t-il, dans ce document,
> une valeur corrigée par ce commit qui subsiste sous sa forme fausse ailleurs ?*
>
> **« Aucun écart » est la réponse attendue si elle est vraie et vérifiée par les commandes
> ci-dessus. N'invente rien pour remplir.**

## Critère de clôture

La passe 3 a été la première à ne **déplacer aucun lieu où le code va s'écrire** — le critère que
les passes 1 et 2 avaient fait échouer. Si la passe 4 ne trouve aucun résidu mécanique, la boucle
se clôt et la story part en `dev-story`.
