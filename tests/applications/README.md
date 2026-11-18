# Application composition corpus

Four authored tasks exercise the actual document loader, tree/bytecode evaluators
and interactive history API. Each has separate Python and Node implementations
of the producer and consumer. `CASES.json` is the eight-case regression subset;
the reviewer driver generates 80 size/seed/direction instances before execution.
They are four application families, not 80 independent real-user applications.

- Invoice ledger: discard void rows, first-posted invoice deduplication, integer
  cents/discount accounting, signed refunds with floor division, regional totals.
- Routes: minimum nonnegative parallel edge, zero-cost edges, disconnected nodes
  and weighted single-source distances. Python uses heap-based Dijkstra, Node a
  linear frontier; independent reference uses SQL and all-pairs Floyd-Warshall.
- Concordance: NFC plus **ASCII** lowercase, explicit space/tab/LF/CR/NBSP
  separators, occurrence/document frequencies, UTF-8 byte order. This is not
  Unicode full case folding. Wide IDs remain strings; special object-key text,
  combining marks, BMP and astral characters are included.
- Wine: UCI red-wine CSV, exact decimal density/pH scaling to integers and
  quality-group count/sum/min/max. Full input contains 1599 rows; no predictor
  accuracy or general machine-learning performance is claimed.

Independent SQL/Fraction/Floyd references live in the reviewer experiment
`application_cases.py`; no foreign program output supplies expected answers.
The raw/public-data digest, citation, license and retrieval source are preserved
in `data/PROVENANCE.json`. Wine Quality is by Cortez, Cerdeira, Almeida, Matos and
Reis (2009), UCI, DOI 10.24432/C56S3T, CC BY 4.0. CSV text fixture generation
normalizes physical line endings and uses the requested first N data rows.

The composition copies input into a producer, gets a normalized value, copies it
into the consumer, records an expected ordinary exception after saving `answer`,
recovers it, replaces the consumer's bridge with null, verifies the producer's
copy and answer remain, and closes both sessions. Interactive recordings repeat
these actions and strictly replay both raw and losslessly packed history.
Native and independent checks distinguish explicit null from display stdout.
