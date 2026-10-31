# Story v017-4a : factures fournisseur — brouillon, validée, payée — le backend

Status: ready-for-dev

*Sous-fiche issue du découpage de la Story v017-4 à sa rédaction (2026-10-31, C-v017-4-1). Worktree
`/home/gcorbaz/devel/kesh-0175`, branche `story/v017-4-facture-fournisseur-brouillon` (partie de `8e126cfd`) ; la 4a
d'abord, puis la 4b. Registre **commun** : `v017-4-choix-autonomes.md` (C-v017-4-1 à 20 ; C-v017-4-21 et suivants au
développement). Issues : **#849**, #853 — en `refs` seulement. **Le texte des AC, le contexte, les décisions, le
contrat de l'API, les codes d'erreur, la précédence, les verrous, l'inventaire clos, les limites et les questions au
Project Lead sont dans la fiche parente `v017-4-facture-fournisseur-brouillon.md`** : ils valent ici sans être recopiés.
Les numéros de ligne cités sont ceux de `8e126cfd`.*

## Story

En tant que Guy, je veux que l'API de Kesh tienne le cycle brouillon → validée → payée d'une facture fournisseur — et
que la 0.17.0 monte sans perte une base 0.16.x ou rc1 peuplée —, afin que l'écran de la `v017-4b` n'ait plus qu'à le
présenter.

## Périmètre

`kesh-db` (migration `20261031000001_supplier_invoices_draft`, dépôt des factures fournisseur, K8, `delete_in_tx` —
doc-comment —, lettrage et lots — types), `kesh-api` (routes `PUT`, `DELETE`, `validate`, `unvalidate`, filtre de la
liste, création et complétion en brouillon, règles « document » extraites, erreurs, audit, MCP, export CSV, registres),
`kesh-i18n` (messages d'erreur, libellés d'audit, quatre catalogues), `kesh-seed`, `docs/api-external.md`,
`docs/migrations-idempotence-audit.md`, les tests Rust, et les textes liés à la migration : CHANGELOG (en-tête et
*Modifié*), manuel d'administration et son PDF. **Livrable seul** : le cycle se teste de bout en bout en HTTP.

## Critères d'acceptation

**AC-1 à AC-20** de la fiche parente, sous leur numéro d'origine. Mutations **M1 à M20**, dont M12a à M12c et M20-bis à
M20-quater (validation P1). **Dérogation écrite** (fiche parente, § *Découpage*, C-v017-4-31) : la 4a ne se publie
qu'avec la 4b, sur la même branche.

## Tâches

- [ ] Étape 0 — relire au code les prémisses citées par la fiche parente (§ *Ce que le code dit*, § *Inventaire
      clos*) ; tout écart au Change Log, ligne lue citée.
- [ ] Tests rouges d'abord (commit séparé, sortie rouge au Dev Agent Record) : AC-1 (contraintes), AC-2 (deux montées),
      AC-3, AC-5 à AC-16.
- [ ] Migration (dont la colonne `link_origin`, et le repli du `MODIFY COLUMN` joué deux fois), P5 à P8, squash,
      `migrations.sha384` (AC-1).
- [ ] Entité et lecteurs de l'écriture d'achat (AC-4) ; inventaire clos tenu au Dev Agent Record.
- [ ] Dépôt : `create_draft_in_tx`, `update_draft`, `delete_draft`, `validate_in_tx`, `unvalidate`, `create_validated*`
      composé (AC-6 à AC-12) ; tête `SupplierInvoiceDraft` (AC-13).
- [ ] Routes, règles « document » extraites, #853 serveur, filtre de liste (AC-6 à AC-11, AC-14, AC-15).
- [ ] K8 (AC-16) ; MCP (AC-17) ; audit, registres, messages (AC-18).
- [ ] Appelants des tests et du seed (AC-12).
- [ ] `api-external.md` (AC-19) ; CHANGELOG, manuel d'administration et PDF (AC-20).
- [ ] Mutations M1 à M20.
- [ ] Gate complet backend (base reconstruite), Vitest et **E2E ciblés** de ses routes au dernier commit de code de la
      4a ; l'**E2E complet** se joue au dernier commit de code de la **4b** (dérogation, C-v017-4-31) — le Dev Agent Record
      le dit ainsi ; décomptes recomptés.

## Dev Agent Record

*(À remplir au développement.)*

## Change Log

- 2026-10-31 — Sous-fiche créée au découpage de la v017-4 (C-v017-4-1). Prochaine étape : validation P1 de la fiche
  parente et des deux sous-fiches (`v017-4-validate-prompt-p1.md`).
- 2026-10-31 — Validation P1 remédiée dans la fiche parente (Change Log de la parente ; C-v017-4-26 à 33) : K8 en dernier,
  total du brouillon en `kesh-core`, régime de verrous et de rejeu, règles de la complétion au `PUT` selon `link_origin`,
  `sourceDocument` enrichi, dérogation E2E, règle « Validée » aux textes de la 4a. Prochaine : validation P2.
