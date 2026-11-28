# Validation P1 de la fiche 49-2-reordonnancement-lignes — prompt de la passe

*Passe complète, contexte frais, **lecture seule**. Modèle : **Sonnet** (la fiche est d'Opus 5.5 ; une première passe n'est
jamais ciblée et ne va jamais à Haiku — CLAUDE.md § *Review Iteration Rule*).*

Tu valides la fiche de la story **49-2-reordonnancement-lignes** de Kesh (release v0.21.0, vendredi 2026-12-04 ; Epic 52
réduit ; #1242). Kesh est un logiciel de comptabilité suisse **en production** : Guy y tient ses livres réels et émet de
vraies factures. Français soigné. Tu ne corriges rien : tu rends des findings.

- Worktree : `/home/gcorbaz/devel/kesh-49-2`, branche `story/49-2-reordonnancement-lignes`.
- Fiche : `_bmad-output/implementation-artifacts/49-2-reordonnancement-lignes.md`. Code de référence : **`0e99cd2db`**
  (`main`, la v0.20.0) ; depuis, seuls des fichiers de planification ont été commités (fichiers d'epic, `sprint-status.yaml`,
  `README.md`, la fiche et ce prompt) — vérifie-le : `git -C /home/gcorbaz/devel/kesh-49-2 diff --stat 0e99cd2db HEAD`.
- Rapport à écrire : `/home/gcorbaz/devel/kesh-gate-logs/e492/validation-49-2-p1.md` (crée le dossier s'il manque).

## À lire d'abord

1. `CLAUDE.md` du worktree : *Review Iteration Rule* (dont *Inventorier les sites NON RÉSOLUS*, *Une passe déclare ses
   axes exercés*, *Le prompt d'une passe doit NOMMER le manuel*), *Recette visuelle*, *Test Locally First*, *Ce qu'impose
   la production*, *Règle de splitting préventif*, *Migration breaking policy*, *Numérotation des versions*.
2. `_bmad-output/planning-artifacts/epic-52-lignes-de-facture-suite.md`, `epic-62-lignes-de-facture-titres.md`, et
   `epic-49-types-de-lignes-facture.md` (§ « Ce qui se lit au code », § 49-2, tableau des fichiers).
3. La fiche close `_bmad-output/implementation-artifacts/49-1c-types-de-lignes-ecran.md` (mapper `invoice-lines.ts`,
   pièges, Dev Notes qui renvoient à la 49-2).
4. L'issue, en lecture : `gh issue view 1242 --json number,title,labels,milestone,body` (et #1330, #1363, #1400 pour les
   frontières).

## Axes — chacun à déclarer exercé ou non exercé

1. **Périmètre contre le fichier d'épique et #1242** : ↑ / ↓ en brouillon, accessibles au clavier, E2E « créer toutes les
   lignes, puis réordonner », i18n, manuel ; le refus du glisser-déposer (D3) est-il fondé ? Rien n'est pris à la 49-3 ni à
   la 49-4b ; les avoirs sont-ils vraiment hors sujet (D9 : pas de brouillon d'avoir) ?
2. **Prémisses relues au code** : chaque citation `chemin:ligne` de § *Ce qui se lit au code* et de § *Inventaire* se relit
   à `0e99cd2db` (`git -C /home/gcorbaz/devel/kesh-49-2 show 0e99cd2db:<chemin> | sed -n '<l>p'`) ; une ligne fausse ou un
   fait faux est un finding. Vérifie en particulier : la position est l'indice dans le vecteur reçu (`insert_lines`) ; le
   `DELETE` précède l'insertion dans la même transaction (D10, pas de migration) ; `is_no_op_change` compare dans l'ordre ;
   l'écriture de vente agrège par `BTreeMap` (AC9) ; l'avoir copie la position ; aucune route ne modifie un avoir ; la page
   d'édition ne contrôle pas le statut.
3. **Inventaire par le symptôme** : rejoue S1 et S2 ; cherche, **en partant du symptôme** (un ordre de lignes perdu ou qui
   change ce qu'il ne doit pas), d'autres sites : relectures de `invoice_lines` / `credit_note_lines` sans `ORDER BY`
   (MCP, exports, courriel, rappels, PDF de l'avoir, règlements), tris côté frontend, calculs dépendants de l'ordre
   (arrondi, capacité du PDF de la 49-4a, récapitulatif TVA). Tout site hors de la fiche, sans issue, est un finding.
4. **Critères d'acceptation constatables** : chaque AC (AC1 à AC14) se constate par un test, une commande ou une capture
   nommés ; un AC flou est un finding. AC11 (aucun code de production sous `crates/`) est-il compatible avec les tâches ?
5. **Preuve du rouge** : chaque test neuf (T1 à T7) a une mutation qui le fait rougir sans le correctif, et que le test, tel
   que décrit, tuerait réellement. En particulier : T3 rougit-il sur `ORDER BY … DESC` à la ligne citée (laquelle des deux
   relectures `update` rend-il) ? T4 compare-t-il deux écritures produites par deux ordres différents, et `ecriture()`
   (`invoice_line_types_e2e.rs:815`) est-elle sensible à l'ordre des lignes d'écriture ? T5 : quel code rend le `PUT`
   quand seule la garde de statut est retirée ? T2 peut-il observer le focus dans jsdom ?
6. **Factures réelles en production** : la story peut-elle changer une facture ou un avoir existants (lignes, positions,
   totaux, TVA, écritures, PDF figés ou non) ? Un brouillon rouvert et enregistré sans geste envoie-t-il bien le même corps
   (test « AC9 a ») ? Un chemin non couvert est un finding.
7. **Décisions** : chaque D (D1 à D12) a son option écartée et son motif ; juge sur le fond D1 (pas de route dédiée), D2
   (snippet commun plutôt qu'une colonne), D3 (pas de glisser-déposer), D5 (focus et annonce), D7 (effacement du bandeau),
   D11 (no-op de deux lignes identiques).
8. **Accessibilité** : boutons natifs, noms accessibles renumérotés, focus après déplacement et à la borne, zone
   `aria-live` présente avant le premier déplacement, états désactivés ; un manque est un finding.
9. **Manuel, PDF compris** : `docs/manual/fr/user-manual.tex` (`:1572` et toute autre phrase sur l'ordre des lignes, la
   modification d'un brouillon, la dévalidation) et `docs/api-external.md` (§ des lignes d'une facture client, § `PUT
   /invoices/{id}`) ; **aplatis aussi le PDF** avant de greper (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' |
   tr -s ' '`, idem `admin-manual.pdf`) et cherches-y ce que la story rendra faux ou devra dire (« ordre des ajouts »,
   « fin de tableau », lignes de facture, avoir). Un texte qui deviendra faux sans tâche dans la fiche est un finding.
10. **E2E, recette visuelle et gates** : T6 joue le parcours au clavier dans un vrai navigateur ; les captures C1 à C6
    couvrent chaque état touché, sur une base de démonstration ; les gates suivent le `CLAUDE.md` (exception `kesh-db` : la
    story touche un fichier de dépôt ; `gate-ci` et E2E complet au dernier commit de code ; bases `kesh_e492`,
    `kesh_ci_e492`, `kesh_e2e_e492`).
11. **Splitting et migration** : le compte de deux modules de production est-il juste (CLAUDE.md : un module est un crate
    ou un domaine du frontend dont le code de production change ; les tests ne comptent pas) ? La story n'exige-t-elle
    vraiment aucune migration ?
12. **Coordination** : fichiers communs avec l'Epic 62 (49-3 : `InvoiceForm.svelte`, `user-manual.tex`, `api-external.md`)
    et avec les autres epics de la v0.21.0 (catalogues i18n, `sitesTotal`).

## Interdits

Lecture seule. **Aucune écriture** hors de ton rapport (`/home/gcorbaz/devel/kesh-gate-logs/e492/validation-49-2-p1.md`).
Aucun `cargo`, `npm`, `npx`, `make`, `docker`, `docker compose`, `mariadb`, `sqlx` ; aucun script de `scripts/` — ni
`scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/recette-montee.sh`, ni
`scripts/mem-guard.sh`, ni aucun script mutant ; aucun `git commit`, `git checkout`, `git switch`, `git reset`, `git stash`,
`git push`, `git merge`, `git rebase`, `git worktree` ; aucun `gh` qui écrit (`gh issue create|edit|comment|close`,
`gh pr …`, `gh api` en `POST`/`PATCH`/`DELETE`). Jamais `pkill`, `killall` ni `kill` par motif. Rien sous `~/kesh/`
(production, lecture seule). Aucune donnée réelle de Guy. Ne touche à aucun autre worktree.

## Rapport

En tête : **verdict chiffré** `CRITICAL / HIGH / MEDIUM / LOW` (par exemple `0/0/2/5`). Puis chaque finding : identifiant
(`V1-n`), sévérité, axe, constat **avec la ligne lue citée** (`chemin:ligne` et la ligne copiée, ou la commande et sa
sortie), conséquence, remède proposé. Puis la liste des **axes exercés** (pour chacun : commandes lancées, fichiers lus) et
des **axes non exercés** (avec la raison). Un « 0 finding » sans cette liste ne compte pas comme passe. Pour tout finding
CRITICAL ou HIGH qui affirme l'absence d'un code ou la présence d'un anti-pattern, cite le `grep -nF` qui l'établit.
