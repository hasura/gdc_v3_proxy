# GDC v3 Proxy

This is an implementation of the Graphql Data Connector spec for hasura v2.

The underlying source is any server implementing the Graphql Data Connector spec for hasura v3.

Note this is very much a work in progress, and should be considered highly experimental.

The endpoint for the underlying target connector should be provided as the query parameter `proxy_target_url`

If this container is deployed at `http://proxy.com` and your target connector is deployed at `http://target.com`, the url to specify when using this conector would be `http://proxy.com?proxy_target_url=http://target.com`

One implication of this is that you would need to add the connector multiple times if you have multiple data sources, and you cannot reuse the same connector for multiple databases.

Of course this is only true as far as HGE is concerned, and nothing stops you from using the same deployed proxy for multiple underling connectors, presuming you configure those correctly.
