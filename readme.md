# GDC v3 Proxy

This is an implementation of the Graphql Data Connector spec for hasura v2.

The underlying source is any server implementing the Graphql Data Connector spec for hasura v3.

Note this is very much a work in progress, and should be considered highly experimental and unfinished.

Due to some technical limitations, the endpoint for the underlying data connector must be provided as an env var or flag.

This is a departure from the usual way of configuring v2 connectors, but is necessary for technical reasons.

Currently, the same endpoint should also be provided as configuration. This is redundant and may change later.
