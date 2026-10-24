# Prompt — validation P3, Story 32-4a (la couche de services : rollout des lectures — écritures, postes ouverts, contacts)

*Versionné le 2026-10-24. Passe **ciblée** (troisième passe) : **une seule lentille, regression hunter**, en contexte
frais, **Sonnet** (la P2 était Opus ×2). Lecture seule stricte. Motif de la passe ciblée (`CLAUDE.md`, *La passe
ciblée*) : la P2 a rendu un seul MEDIUM (R3), **né de la remédiation P1** ; ce qu'il reste à relire n'est plus la fiche
entière, c'est la remédiation P2 — et la règle de garde qu'elle a changée.*

⛔ **La boucle ne se clôt que sur une passe qui ne rend AUCUN finding au-dessus de LOW** (`CLAUDE.md`, *Review
Iteration Rule*, décision du 2026-10-23). Un MEDIUM, même sur la seule rédaction de la fiche, appelle une passe de plus.
Ne baisse pas une sévérité pour clore la boucle ; ne la hausse pas pour la prolonger.

Dépôt `/home/gcorbaz/devel/kesh-32-4a`, branche `story/32-4a` (partie de `main` `89d68704`). **Périmètre** :

1. **le diff de la remédiation P2** : `git show cbcd3237` (fiche
   `_bmad-output/implementation-artifacts/32-4a-services-ecritures-contacts.md`, registre
   `epic-32-choix-autonomes-324a.md`, ligne de `sprint-status.yaml`) ;
2. **l'AC12 entier** de la fiche (la garde lexicale : définition du corps, détecteur aplati, (ii), (iii) et son
   inventaire au niveau du fichier, (vi), (viii), limites), et ce qui s'y rattache : M15, M16, M16′ de l'AC14, l'AC17.

Rapports de la P2, pour savoir ce qui a été corrigé et pourquoi : `/home/gcorbaz/devel/kesh-gate-logs/324a-validate-p2-R.md`
et `324a-validate-p2-F.md` ; leur sort est au Change Log de la fiche (« validation P2 »). Code de référence :
`crates/kesh-api/tests/services_motif.rs` (`sans_commentaires` l. 1998, `corps_des_handlers` l. 2009-2033,
`ac10_i_ii_handlers_sans_le_pool` l. 2073-2100), `crates/kesh-api/src/routes/{journal_entries,letterings,contacts,accounts}.rs`,
`crates/kesh-db/src/repositories/` et `util/search.rs` pour la fermeture des appels du fait 6.

## Lentille — regression hunter

La sévérité se déplace vers ce qu'on vient d'écrire. Cherche ce que la remédiation P2 a **cassé, contredit ou laissé
faux**, et ce que la règle neuve de l'AC12 a d'inatteignable. Pour chaque affirmation, ouvre le site (`sed -n`,
`grep -nF`) et **copie la sortie**. Axes, tous à exercer :

1. **La définition unique du corps (AC12)** : est-elle énoncée une fois et employée partout de la même façon dans la
   fiche (AC12, AC13 « même définition qu'à l'AC12 », AC4, Change Log P2) ? Concorde-t-elle avec `corps_des_handlers`
   tel qu'il est (ouverture `starts_with`, fin à la première ligne égale à `}`) ? La (ii) de `list_accounts` « s'y
   ramène »-t-elle sans rien perdre (lis `accounts.rs` entre la fin de `list_accounts` et le `pub async fn` suivant) ?
   Le choix par **nom exact** est-il nécessaire et suffisant (préfixes : `get_lettering`, `get_contact`,
   `get_journal_entry`, `list_contacts`) ?
2. **L'inventaire au niveau du fichier (AC12 (iii))** : recompte toi-même, sur les trois fichiers tronqués au
   `#[cfg(test)]`, commentaires écartés, aplatis, les occurrences de `\bstate\s*\.\s*pool\b` sur le fichier et sur les
   corps — 15 / 10 / 12 des deux côtés ? Un faux rouge aujourd'hui (une occurrence hors corps, un `/*`, une chaîne
   contenant `//` qui couperait différemment fichier et corps) ? Un faux rouge **après** un correctif conforme (ce que
   les handlers migrés, les DTO, `contact_response_with_label`, `parse_as_of` déplacée laissent dans les fichiers) ?
   Le compte voit-il bien un `pub(crate) async fn` (M16′) et une aide synchrone hors corps — et un faux vert reste-t-il
   possible **sous le nom `state`** (une aide placée **dans** un corps ? une macro ?) ? La suppression du compte des
   `async fn` toutes visibilités (P1) laisse-t-elle un cas que lui seul tuait ?
3. **Les limites écrites** : la liste « les limites qui restent » est-elle juste et close pour ce que le détecteur
   prétend voir — ou une forme qu'elle omet (sous le nom `state`) ? L'exclusion d'`accounts.rs` est-elle juste
   (`response`, l. 185-190) et sa conséquence dite ? Cohérente avec l'AC17, le *Hors périmètre* (#738 B4) et C-32-4a-10 ?
4. **M16′ et l'AC14** : M16′ est-elle tuée par l'inventaire au niveau du fichier, telle qu'écrite ? Le décompte
   « 35 — 34 qui doivent rougir, et M8, équivalente déclarée » est-il juste ? M14 réécrite (`map_err(kesh_db::errors::
   map_db_error)?`) compile-t-elle dans un service (chemin, `From<DbError> for AppError`) et tue-t-elle l'AC11 (a) ?
5. **Les corrections de fait de la P2** (R1, R2, R4-R8, F3-F7) : chaque ligne citée neuve est-elle exacte
   (`imported_supplier_invoices.rs:373-378`, `:386-395` ; la troisième puce du fait 6 et ses lignes ; les `#[cfg(test)]`
   de l'AC13 ; `plan_comptable.rs:35-36` ; `contacts.rs:584-585` ; `api-external.md:426-433` ;
   `kesh-db/.../journal_entries.rs:775-776`, `:934` ; `services_motif.rs:43-141`) ? La définition « toute fonction ou
   `const` atteinte depuis un appel direct » rend-elle le tri de l'AC13 décidable, ou en change-t-elle le résultat
   attendu (une occurrence d'écriture ou de `FOR UPDATE` désormais **dans** la fermeture) ?
6. **Propagation et historique** : une valeur corrigée par la P2 restée ancienne dans une entrée **vivante** de la
   fiche, du registre ou de `sprint-status.yaml` ? Une entrée **datée** (fiche rédigée, validation P1) réécrite au lieu
   d'annotée ? L'annotation datée de F3 (texte d'issue, `%2B10000-01-01`, #742) est-elle exacte ? Le Change Log P2
   est-il cohérent avec les rapports P2 (15 findings, 13 distincts, 1 MEDIUM, 12 LOW, recoupements R1 = F1, R2 = F2) ?

## Ce que tu rends

Findings `P3-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et sortie
copiée), correction proposée ; dis si chacun naît de la **remédiation P2** (`cbcd3237`) ou d'avant. Un bilan d'en-tête
**cohérent avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (1 à 6) — sans elle, « 0 finding » ne compte
pas comme passe. Ce que la fiche **prescrit** au développement n'existe pas encore : son absence n'est pas un défaut.
Rapport écrit **d'un seul jet**, un seul fichier : `/home/gcorbaz/devel/kesh-gate-logs/324a-validate-p3.md`. Dernier
message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`,
`docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors
de ton périmètre. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `gh issue view|list`,
`pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`, `head`, `awk`, `wc`, `uniq` vers la sortie
standard.
