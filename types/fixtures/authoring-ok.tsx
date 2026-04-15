import { eventValue, render, useState } from "egui";
import { motion } from "motion/react";

function App() {
  const [open, setOpen] = useState(false);

  return (
    <div id="root" data-slot="column" className="gap-2">
      <button
        id="toggle"
        data-slot="button"
        variant="primary"
        label={open ? "Open" : "Closed"}
        onClick={() => setOpen((value) => !value)}
      />
      <input
        id="open-toggle"
        type="checkbox"
        checked={open}
        onToggle={(event) => setOpen(Boolean(eventValue<boolean>(event)))}
      />
      <motion.label
        id="status"
        text="Status"
        animate={{ opacity: open ? 1 : 0 }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
      />
    </div>
  );
}

render(<App />);
