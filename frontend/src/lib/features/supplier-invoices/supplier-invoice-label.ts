/**
 * Le libellé d'une facture fournisseur — Story 33-1b (#926). Une seule définition côté
 * écran, miroir de `kesh_core::invoice_format::supplier_invoice_{short,long}_label`.
 *
 * - **forme courte** : ce qui nomme la facture dans une phrase ;
 * - **forme longue** : ce qui sert à reconnaître la pièce (`FF-2026-001 (ABC-123)`).
 *
 * Un numéro vide ou blanc vaut absent (règle de K8, `closing_checks.rs`).
 */

/** Les deux numéros d'une facture fournisseur, sous leurs noms d'API. */
export interface SupplierInvoiceNumbers {
	id: number;
	internalNumber?: string | null;
	supplierInvoiceNumber?: string | null;
}

/**
 * Le numéro sans ses blancs de bord, `null` s'il n'en reste rien. Ce `trim` n'est pas tout à
 * fait celui de Rust ni celui de MariaDB : voir `non_blank`, `kesh_core::invoice_format`.
 */
function nonBlank(numero: string | null | undefined): string | null {
	const net = numero?.trim();
	return net ? net : null;
}

/**
 * Le numéro à montrer, ou `null` : le numéro interne, à défaut celui du fournisseur. Pour les
 * sites qui ont leur propre repli (`documentLabel`, `groupDocument`).
 */
export function supplierInvoicePreferredNumber(
	interne: string | null | undefined,
	fournisseur: string | null | undefined,
): string | null {
	return nonBlank(interne) ?? nonBlank(fournisseur);
}

/** La forme courte : le numéro interne, à défaut celui du fournisseur, à défaut `#id`. */
export function supplierInvoiceShortLabel(f: SupplierInvoiceNumbers): string {
	return supplierInvoicePreferredNumber(f.internalNumber, f.supplierInvoiceNumber) ?? `#${f.id}`;
}

/** La forme longue : `FF-2026-001 (ABC-123)`, ou un seul des deux, à défaut `#id`. */
export function supplierInvoiceLongLabel(f: SupplierInvoiceNumbers): string {
	const interne = nonBlank(f.internalNumber);
	const fournisseur = nonBlank(f.supplierInvoiceNumber);
	if (interne && fournisseur) return `${interne} (${fournisseur})`;
	return interne ?? fournisseur ?? `#${f.id}`;
}
