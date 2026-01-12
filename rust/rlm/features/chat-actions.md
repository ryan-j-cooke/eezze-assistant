### Actions Feature (Summary)

The Actions feature allows the agent to **request structured interactions from the user** instead of only returning free-form text. Each action defines a **type**, a **message/prompt**, and optional **parameters or choices**. Key points:

1. **Structured Output**  
   - Agent emits JSON-like action objects, e.g.:  
     ```json
     {
       "type": "confirmation",
       "message": "Do you want to apply this change?",
       "options": ["yes", "no"]
     }
     ```

2. **User Interaction**  
   - Frontend (e.g., VS Code extension) interprets the action.  
   - Displays UI elements like modals, choice menus, or input boxes.  
   - Requires user input to proceed if marked as mandatory.

3. **Control Flow Enforcement**  
   - The agent pauses reasoning until user input is received.  
   - Allows safe, interactive workflows for sensitive operations (e.g., file changes, deletions, refactors).

4. **Extensibility**  
   - Supports multiple action types: confirmation, choice selection, text input, tool invocation, etc.  
   - Acts as a **protocol between agent and frontend**, enabling deterministic, interactive behavior.

> **Essentially:** Actions are a structured mechanism for agents to request explicit user confirmation or input, enabling safe, controlled, and interactive decision-making.
