"""Join independent facts for one real request submitted through Ask alo.

Native input and settings belong to the runtime. This coordinator never sends
an approval or changing recorder call and never restores the access policy.
"""

from asking_turn_observation import AskingTurnObservation
from delivery_observation import DeliveryObservation
from record_delta import snapshot, verify_microphone_change, verify_microphone_refusal


def check_microphone_turn(runtime, *, enabled, full_access, withdraw=False):
    """Observe approved enable, withdrawn disable or Full disable in that order."""
    cases = {(True, False, False): 'enable', (False, False, True): 'withdraw',
             (False, True, False): 'full'}
    if (any(type(value) is not bool for value in (enabled, full_access, withdraw))
            or (enabled, full_access, withdraw) not in cases):
        raise ValueError('This private walk supports only its three microphone boundaries')
    kind = cases[enabled, full_access, withdraw]
    asking = delivery = failure = result = None
    try:
        runtime.require_status(not enabled)
        checkpoint = snapshot(runtime.record_path)
        delivery = DeliveryObservation()
        asking = AskingTurnObservation()
        runtime.request(kind)
        card = None
        if full_access:
            asking.capture_started()
        else:
            card = runtime.wait_pending(enabled, asking, delivery)
            asking.capture_pending()
            runtime.card_ready(kind, card)
            if withdraw:
                runtime.withdraw(card)
        turn = asking.finish()
        runtime.turn_exited(turn)
        if withdraw:
            delivered = delivery.finish_refused()
            recorded = verify_microphone_refusal(runtime.record_path, checkpoint, enabled=enabled)
        else:
            delivered = delivery.finish(enabled=enabled)
            recorded = verify_microphone_change(runtime.record_path, checkpoint, enabled=enabled,
                from_approval=None if full_access else card['number'], full_access=full_access)
        runtime.require_status(not enabled if withdraw else enabled)
        result = dict(kind=kind, pending=card, turn=turn, delivery=delivered, record=recorded)
    except BaseException as error:
        failure = error
    finally:
        for observer in (asking, delivery):
            if observer is not None:
                try:
                    observer.close()
                except BaseException as error:
                    if failure is None:
                        failure = error
                    else:
                        failure.add_note(f'Additional observer cleanup failure: {error}')
    if failure is not None:
        raise failure
    return result
