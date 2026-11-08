import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, expect, test, vi } from "vitest";
import two from "./fixtures/two.json";
import type { Row } from "./types";
import MetaPanel from "./MetaPanel.svelte";

// Vitest runs without globals here, so testing-library cannot unmount between tests by itself.
afterEach(cleanup);

const row: Row = {
  ...two.rows[1],
  title: "Old Title",
  year: "",
  author: "",
  note: "",
  group: "Games",
};

async function commit(label: string, value: string) {
  await fireEvent.click(
    screen.getByRole("button", { name: new RegExp(`edit ${label}`, "i") }),
  );
  const input = screen.getByRole("textbox", { name: new RegExp(label, "i") });
  await fireEvent.input(input, { target: { value } });
  await fireEvent.keyDown(input, { key: "Enter" });
}

test("AC-159 (RENDER): the selected entry shows its five fields, each change one setMeta edit", async () => {
  const onedit = vi.fn();
  render(MetaPanel, { props: { row, editable: true, onedit } });

  for (const label of ["Title", "Year", "Author", "Note", "Group"]) {
    expect(
      screen.getByRole("button", { name: new RegExp(`edit ${label}`, "i") }),
    ).toBeInTheDocument();
  }
  expect(screen.getByText("Old Title")).toBeInTheDocument();
  expect(screen.getByText("Games")).toBeInTheDocument();

  await commit("Title", "New Title");
  expect(onedit).toHaveBeenLastCalledWith({
    kind: "setMeta",
    index: row.index,
    field: "title",
    value: "New Title",
  });

  await commit("Year", "1985");
  expect(onedit).toHaveBeenLastCalledWith({
    kind: "setMeta",
    index: row.index,
    field: "year",
    value: "1985",
  });

  await commit("Group", "");
  expect(onedit).toHaveBeenLastCalledWith({
    kind: "setMeta",
    index: row.index,
    field: "group",
    value: "",
  });
  expect(onedit).toHaveBeenCalledTimes(3);

  // An unchanged field is not an edit.
  await commit("Author", "");
  expect(onedit).toHaveBeenCalledTimes(3);
});

test("AC-159 (RENDER): on a view-only cart the fields are shown, not offered for editing", () => {
  render(MetaPanel, { props: { row, editable: false, onedit: vi.fn() } });
  expect(screen.getByText("Old Title")).toBeInTheDocument();
  expect(screen.getByText("Games")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /edit title/i })).toBeNull();
});
