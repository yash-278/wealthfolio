import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { BedrockRegionField } from "./bedrock-region-field";

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));

describe("Bedrock region settings", () => {
  it("preserves the region from a previously saved Runtime URL", () => {
    const save = vi.fn();
    render(
      <BedrockRegionField
        url="https://bedrock-runtime.us-east-1.amazonaws.com/openai/v1"
        onSave={save}
      />,
    );
    expect(screen.getByRole("textbox")).toHaveValue("us-east-1");
    fireEvent.click(screen.getByRole("button"));
    expect(save).toHaveBeenCalledWith("https://bedrock-mantle.us-east-1.api.aws/v1");
  });
  it("rejects invalid regions without saving", () => {
    const save = vi.fn();
    render(<BedrockRegionField onSave={save} />);
    for (const value of ["", "https://evil.test", "us-east-1/evil", "us-east-", "US-EAST-1"]) {
      fireEvent.change(screen.getByRole("textbox"), { target: { value } });
      expect(screen.getByRole("button")).toBeDisabled();
    }
    expect(save).not.toHaveBeenCalled();
  });

  it("requires a region and saves the endpoint, without a default", () => {
    const save = vi.fn();
    render(<BedrockRegionField onSave={save} />);
    const button = screen.getByRole("button");
    expect(button).toBeDisabled();
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "eu-west-1" } });
    fireEvent.click(button);
    expect(save).toHaveBeenCalledWith("https://bedrock-mantle.eu-west-1.api.aws/v1");
  });
});
