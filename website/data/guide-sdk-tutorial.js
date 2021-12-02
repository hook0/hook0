module.exports = [
  {
    language: "Python",
    sdk: {
      repository: "",
      setup: "pip install hook0"
    },
    send_message: `hook0 = Hook0("AUTH_TOKEN")
hook0.message.create(
    "c0ea6ffa-1972-4435-b434-ec9e93d38f42",
    MessageIn(
        event_type: "invoice.paid",
        event_id: "evt_Wqb1k73rXprtTm7Qdlr38G",
        payload: {
            "id": "invoice_WF7WtCLFFtd8ubcTgboSFNql",
            "status": "paid",
            "attempt": 2
        }
    )
)`
  },
  {
    language: "NodeJS",
    sdk: {
      repository: "",
      setup: "npm install hook0",
      send_message: `const hook0 = Hook0("AUTH_TOKEN")
await hook0.message.create(
    "c0ea6ffa-1972-4435-b434-ec9e93d38f42",
    {
        event_type: "invoice.paid",
        event_id: "evt_Wqb1k73rXprtTm7Qdlr38G",
        payload: {
            "id": "invoice_WF7WtCLFFtd8ubcTgboSFNql",
            "status": "paid",
            "attempt": 2
        }
    }
)`
    }
  }
];
