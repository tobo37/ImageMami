import os
import optax
import kauldron as kd
from gemma import gm


def main():
    os.environ.setdefault("XLA_PYTHON_CLIENT_MEM_FRACTION", "1.0")

    ds = kd.data.py.Tfds('c4/en', split='train', seq_len=2048)
    model = gm.nn.Gemma3_4B(tokens='batch.input')

    loss = kd.losses.SoftmaxCrossEntropyWithIntLabels(
        logits='preds.logits',
        labels='batch.target',
        mask='batch.loss_mask',
    )

    trainer = kd.train.Trainer(
        seed=42,
        workdir='/tmp/ckpts',
        train_ds=ds,
        model=model,
        init_transform=gm.ckpts.LoadCheckpoint(
            path=gm.ckpts.CheckpointPath.GEMMA3_4B_IT,
        ),
        num_train_steps=300,
        train_losses={'loss': loss},
        optimizer=optax.adafactor(learning_rate=1e-3),
    )

    state, aux = trainer.train()
    gm.ckpts.save_params(state.params, '/tmp/my_ckpt/')


if __name__ == '__main__':
    main()
