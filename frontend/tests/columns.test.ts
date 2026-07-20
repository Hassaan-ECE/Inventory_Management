import { beforeEach, describe, expect, it } from "vitest";

import { getVisibleDataColumnCount } from "@/modules/te-test-equipment/lib";
import { INVENTORY_COLUMNS } from "@/modules/te-test-equipment/types";
import {
  COLUMN_VISIBILITY_STORAGE_KEY,
  readColumnVisibility,
} from "@/modules/te-test-equipment/components/shell/helpers";

describe("column visibility persistence", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("repairs stored visibility when every data column is hidden", () => {
    const allHidden = Object.fromEntries(
      INVENTORY_COLUMNS.map((column) => [column.key, false]),
    );
    localStorage.setItem(COLUMN_VISIBILITY_STORAGE_KEY, JSON.stringify(allHidden));

    const visibility = readColumnVisibility();

    expect(getVisibleDataColumnCount(visibility)).toBe(1);
    expect(visibility.serialNumber).toBe(true);
  });
});
