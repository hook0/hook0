// What the generated request layer puts on the wire, and what it does with what comes back.
//
// The generated half is handed a transport and nothing else, so these cases hand it the real one and
// watch a real API answer: the path it interpolated, the query it assembled, the credential it
// carried, the type it read back, and the error value it answered when the answer was a problem.
//
// Nothing here names an operation the API does not declare. What is exercised is reached through the
// generated package, so an operation that stops being declared takes its case with it rather than
// leaving one that compiles against nothing.

package hook0_test

import (
	"errors"
	"testing"

	hook0 "github.com/hook0/hook0-go"
	"github.com/hook0/hook0-go/generated"
)

func applications(api *fakeAPI) *generated.ApplicationsAPI {
	return generated.NewApplicationsAPI(
		hook0.NewTransport(api.baseURL(), token, 0, 0),
	)
}

func anApplication() map[string]any {
	return map[string]any{
		"application_id":  applicationId,
		"name":            "an application",
		"organization_id": organizationId,
		"consumption":     map[string]any{"events_per_day": 12},
		"onboarding_steps": map[string]any{
			"event":        "Done",
			"event_type":   "ToDo",
			"subscription": "ToDo",
		},
		"quotas": map[string]any{
			"days_of_events_retention_limit": 7,
			"events_per_day_limit":           100,
		},
	}
}

func aProblem() map[string]any {
	return map[string]any{
		"id":     "NotFound",
		"title":  "Not found",
		"detail": "This application does not exist.",
		"status": 404,
		"type":   "https://documentation.hook0.com/problems",
	}
}

func TestAGeneratedMethodReadsBackTheTypeTheAPIDeclares(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 200, body: anApplication()})

	application, err := applications(api).Get(bounded(t), applicationId)
	if err != nil {
		t.Fatalf("the operation failed: %v", err)
	}

	if application.ApplicationId != generated.UUID(applicationId) {
		t.Errorf("the application read back carries %q as its identifier", application.ApplicationId)
	}
	if application.Quotas.EventsPerDayLimit != 100 {
		t.Errorf("the quota read back is %d", application.Quotas.EventsPerDayLimit)
	}
	if application.OnboardingSteps.Event != generated.ApplicationInfoOnboardingStepsEventDone {
		t.Errorf("the onboarding step read back is %q", application.OnboardingSteps.Event)
	}
}

func TestAGeneratedMethodFillsThePathAndCarriesTheCredential(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 200, body: anApplication()})

	if _, err := applications(api).Get(bounded(t), applicationId); err != nil {
		t.Fatalf("the operation failed: %v", err)
	}

	request := api.requests()[0]
	if request.method != "GET" {
		t.Errorf("the operation was issued with %s", request.method)
	}
	// The identifier lands in the path rather than staying as the placeholder that named it.
	if request.target != "/api/v1/applications/"+applicationId {
		t.Errorf("the operation reached %q", request.target)
	}
	if carried := request.headers.Get("Authorization"); carried != "Bearer "+token {
		t.Errorf("the request carried %q as its credential", carried)
	}
}

func TestAGeneratedMethodAssemblesTheQueryTheOperationDeclares(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 200, body: []any{}})

	listed, err := applications(api).List(bounded(t), organizationId)
	if err != nil {
		t.Fatalf("the operation failed: %v", err)
	}

	if len(listed) != 0 {
		t.Errorf("an empty listing read back %d applications", len(listed))
	}
	if target := api.requests()[0].target; target != "/api/v1/applications/?organization_id="+organizationId {
		t.Errorf("the operation reached %q", target)
	}
}

func TestAGeneratedMethodReadsAListOfTheTypeTheAPIDeclares(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{
		status: 200,
		body: []any{map[string]any{
			"application_id":  applicationId,
			"name":            "an application",
			"organization_id": organizationId,
		}},
	})

	listed, err := applications(api).List(bounded(t), organizationId)
	if err != nil {
		t.Fatalf("the operation failed: %v", err)
	}

	if len(listed) != 1 || listed[0].Name != "an application" {
		t.Errorf("the listing read back %v", listed)
	}
}

func TestAProblemTheAPIReportsIsAnsweredAsTheValueItNames(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 404, body: aProblem()})

	_, err := applications(api).Get(bounded(t), applicationId)
	if err == nil {
		t.Fatal("a problem the API reported was read as a success")
	}

	if !errors.Is(err, generated.ErrNotFound) {
		t.Errorf("the failure is %v, which is not the problem the API named", err)
	}
	if errors.Is(err, generated.ErrForbidden) {
		t.Error("the failure answers to a problem the API did not name")
	}

	var reported *generated.ProblemError
	if !errors.As(err, &reported) {
		t.Fatalf("the failure is %T, not the one every problem is a kind of", err)
	}
	if reported.Status != 404 {
		t.Errorf("the failure says the API answered %d", reported.Status)
	}
	if reported.Problem == nil || reported.Problem.Detail != "This application does not exist." {
		t.Errorf("the failure does not carry the document the API answered: %+v", reported.Problem)
	}
}

func TestAFailureThatIsNoProblemDocumentIsStillReported(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{
		status: 502,
		body:   "a gateway wrote this, and it is not a problem document",
	})

	_, err := applications(api).Get(bounded(t), applicationId)
	if err == nil {
		t.Fatal("a gateway failure was read as a success")
	}

	var reported *generated.ProblemError
	if !errors.As(err, &reported) {
		t.Fatalf("the failure is %T, not the one every failure is a kind of", err)
	}
	if reported.Status != 502 {
		t.Errorf("the failure says the API answered %d", reported.Status)
	}
	if reported.Problem != nil {
		t.Errorf("a body naming no problem was read as one: %+v", reported.Problem)
	}
	if reported.Kind != "" {
		t.Errorf("a body naming no problem was given the kind %q", reported.Kind)
	}
}

func TestAnIdentifierThatIsNotOneIsRefusedWhileItIsRead(t *testing.T) {
	api := listen(t)
	answered := anApplication()
	answered["application_id"] = "not-an-identifier"
	api.willAnswer(scriptedResponse{status: 200, body: answered})

	if _, err := applications(api).Get(bounded(t), applicationId); err == nil {
		t.Fatal("a document whose identifier is not one was read as if it were")
	}
}

func TestAGeneratedGroupIsBuiltOnTheClientsOwnTransport(t *testing.T) {
	api := listen(t)
	api.willAnswer(scriptedResponse{status: 200, body: anApplication()})

	// The seam is structural: the hand-written transport answers to the interface the generated
	// package declares without either half naming the other.
	group := generated.NewApplicationsAPI(client(api, promptOptions(1)).Transport())

	if _, err := group.Get(bounded(t), applicationId); err != nil {
		t.Fatalf("a group built on the client's own transport failed: %v", err)
	}
}
