use server_catalog::catalog::CatalogItem;
use server_rules::Visible;

fn fake(item: &CatalogItem) -> Visible<'_> {
    Visible { item }
}

fn main() {}
