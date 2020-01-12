# ascii-diagrams

Render a diagram using only ASCII characters.

This is useful to embed diagrams directly as text instead of using images.

The diagram can be expressed in either TOML or JSON, but the underlying structure is the same.

Here's an example JSON diagram that shows how to render a very simple diagram.

```json
{
  "blocks": [
    {
      "text": "zero",
      "position": { "column": -1, "row": -1 }
    },
    {
      "text": "one",
      "position": { "column": 0, "row": -1 }
    },
    {
      "text": "two",
      "position": { "column": 1, "row": -1 }
    },
    {
      "text": "0000",
      "position": { "column": -1, "row": 0 }
    },
    {
      "text": "four",
      "position": { "column": 1, "row": 0 }
    },
    {
      "text": "oooo",
      "position": { "column": -1, "row": 1 }
    }
  ],
  "edges": [
    { "from": "one", "to": "four" },
    { "from": "one", "to": "0000" },
    { "from": "two", "to": "zero" },
    { "from": "oooo", "to": "zero" }
  ]
}
```
