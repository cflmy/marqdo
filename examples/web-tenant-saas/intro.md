# Two-tenant query demonstration

Compare [Acme notes](/api/items?t=acme) with [Beta notes](/api/items?t=beta).
Each response selects only rows belonging to the chosen tenant.
A [request without a tenant](/api/items) is rejected.

This local demonstration permits anonymous tenant selection. It demonstrates
query filtering and does not authenticate a user or prove tenant membership.

Restarting the server retains existing database rows. The initializer adds
the two demonstration notes only when the table is completely empty.
