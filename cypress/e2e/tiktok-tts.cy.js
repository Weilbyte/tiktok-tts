describe("Tik Tok TTS", () => {
  it("should load correctly", () => {
    cy.visit("/");
    cy.contains("0/300");
  });

  it("should disabled when no text is entered", () => {
    cy.visit("/");
    cy.get("#submit").should("be.disabled");
  });

  it("should be on when text is entered", () => {
    cy.visit("/");
    cy.get("#text").type("Lorem ipsum dolor sit amet");
    cy.get("#submit").should("not.be.disabled");
  });

  it("should updated correctly", () => {
    cy.visit("/");
    cy.get("#text").type("Lorem ipsum dolor sit amet");
    cy.get("#charcount").should("have.text", "26/300");
    cy.get("#text").type(" consectetur adipiscing elit");
    cy.get("#charcount").should("have.text", "54/300");
  });

  it("should generate audio correctly", () => {
    cy.visit("/");
    cy.get("#text").type("Lorem ipsum dolor sit amet");
    cy.get("#voice").select("en_us_001");
    cy.get("#submit").click();
    cy.get("#success").should("be.visible");
    cy.get("#generatedtext").should(
      "have.text",
      '"Lorem ipsum dolor sit amet"'
    );
    cy.get("#audio").should("have.attr", "src").and("not.be.empty");
  });
});
