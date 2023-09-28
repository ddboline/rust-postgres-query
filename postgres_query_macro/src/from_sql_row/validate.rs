use super::{ContainerAttributes, PartitionKind, Property};
use syn::Result;

pub(super) fn validate_properties(
    container: &ContainerAttributes,
    props: &[Property],
) -> Result<()> {
    check_split_in_non_split_container(container, props)?;
    check_stride_in_non_exact_container(container, props)?;

    check_merging_container_attributes(container, props)?;
    check_not_key_and_merge(props)?;

    Ok(())
}

fn check_split_in_non_split_container(
    container: &ContainerAttributes,
    props: &[Property],
) -> Result<()> {
    let is_split = is_match!(
        container.partition.as_ref().map(|attr| &attr.value),
        Some(PartitionKind::Split)
    );

    if is_split {
        Ok(())
    } else {
        let split = props
            .iter()
            .flat_map(|prop| prop.attrs.splits.iter())
            .next();

        match split {
            None => Ok(()),
            Some(split) => Err(err!(
                split.span,
                "explicit `split` in a container without the `#[row(split)]` attribute"
            )),
        }
    }
}

fn check_stride_in_non_exact_container(
    container: &ContainerAttributes,
    props: &[Property],
) -> Result<()> {
    let is_exact = is_match!(
        container.partition.as_ref().map(|attr| &attr.value),
        Some(PartitionKind::Exact)
    );

    if is_exact {
        Ok(())
    } else {
        let stride = props.iter().find_map(|prop| prop.attrs.stride);

        match stride {
            None => Ok(()),
            Some(stride) => Err(err!(
                stride.span,
                "explicit `stride` in a container without the `#[row(exact)]` attribute"
            )),
        }
    }
}

fn check_merging_container_attributes(
    container: &ContainerAttributes,
    props: &[Property],
) -> Result<()> {
    if let Some(merge) = container.merge.as_ref() {
        let has_key = props.iter().any(|prop| prop.attrs.key.is_some());
        let has_merge = props.iter().any(|prop| prop.attrs.merge.is_some());

        if !has_key {
            Err(err!(
                merge.span,
                "You need to specify at least one of the container's fields to be a key: `#[row(key)]`"
            ))
        } else if !has_merge {
            Err(err!(
                merge.span,
                "You need to specify at least one of the container's fields to be the merged field: `#[row(merge)]`"
            ))
        } else {
            Ok(())
        }
    } else {
        let key = props.iter().find(|prop| prop.attrs.key.is_some());
        match key {
            None => {},
            Some(key) => return Err(err!(
                key.span,
                "`#[row(key)]` is only available in containers with the `#[row(group)]` or `#[row(hash)]` attributes"
            )),
        }

        let merge = props.iter().find(|prop| prop.attrs.merge.is_some());
        match merge {
            None => Ok(()),
            Some(merge) => Err(err!(
                merge.span,
                "`#[row(merge)]` is only available in containers with the `#[row(group)]` or `#[row(hash)]` attributes"
            )),
        }
    }
}

fn check_not_key_and_merge(props: &[Property]) -> Result<()> {
    props
        .iter()
        .try_for_each(|prop| match (prop.attrs.key, prop.attrs.merge) {
            (Some(key), Some(merge)) => Err(err!(
                key.span.join(merge.span).unwrap_or(key.span),
                "You cannot specify both `#[row(key)]` and `#[row(merge)]` on the same field"
            )),
            _ => Ok(()),
        })
}
