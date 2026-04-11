import { render, useState } from "egui";
import { motion } from "motion/react";

type MotionNodeProps = {
  open: boolean;
};

const transition = { duration: 0.9, ease: "easeInOut" };

function MotionNodes(props: MotionNodeProps) {
  const open = props.open;
  return (
    <>
      <motion.label
        id="motion-opacity"
        text="opacity"
        visible={false}
        initial={{ opacity: 0.22 }}
        animate={{ opacity: open ? 1 : 0.22 }}
        transition={transition}
      />
      <motion.label
        id="motion-translate"
        text="translate"
        visible={false}
        initial={{ x: -70, y: 20 }}
        animate={{ x: open ? 70 : -70, y: open ? -20 : 20 }}
        transition={transition}
      />
      <motion.label
        id="motion-scale"
        text="scale"
        visible={false}
        initial={{ scale: 0.62 }}
        animate={{ scale: open ? 1.35 : 0.62 }}
        transition={transition}
      />
      <motion.label
        id="motion-rotate"
        text="rotate"
        visible={false}
        initial={{ rotate: -0.75 }}
        animate={{ rotate: open ? 0.75 : -0.75 }}
        transition={transition}
      />
    </>
  );
}

function App() {
  const [open, setOpen] = useState(false);
  return (
    <column id="motion-root" gap={10}>
      <label
        id="motion-title"
        text="JSX-authored motion nodes"
        weight="semibold"
      />
      <button
        id="motion-toggle"
        label={open ? "Reset motion" : "Run motion"}
        onClick={() => setOpen((value: boolean) => !value)}
      />
      <MotionNodes open={open} />
    </column>
  );
}

render(<App />);
