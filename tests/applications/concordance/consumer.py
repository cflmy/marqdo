from collections import Counter, defaultdict
occurrences = Counter()
documents = defaultdict(set)
for row in bridge:
    for word in row['tokens']:
        occurrences[word] += 1
        documents[word].add(row['id'])
answer = [{'word':word,'occurrences':occurrences[word],'documents':len(documents[word])}
          for word in sorted(occurrences,key=lambda s:s.encode('utf-8'))]
answer
